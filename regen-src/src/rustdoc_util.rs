use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;

use rustdoc_types::Crate;
use rustdoc_types::Id;
use rustdoc_types::Item;
use rustdoc_types::ItemEnum;
use rustdoc_types::ItemKind;
use rustdoc_types::Module;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct NamedItem<'a, T: Clone + Debug + PartialEq + Eq + Hash + Serialize + Deserialize<'a>> {
	pub name: &'a String,
	pub base: &'a Item,
	pub inner: &'a T,
}

impl<'a, T> Ord for NamedItem<'a, T>
where
	T: Clone + Debug + PartialEq + Eq + Hash + Serialize + Deserialize<'a>,
{
	fn cmp(&self, other: &Self) -> Ordering {
		let name_ordering = self.name.cmp(other.name);
		if name_ordering == Ordering::Equal {
			self.base.id.0.cmp(&other.base.id.0)
		} else {
			name_ordering
		}
	}
}

impl<'a, T> PartialOrd for NamedItem<'a, T>
where
	T: Clone + Debug + PartialEq + Eq + Hash + Serialize + Deserialize<'a>,
{
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a, T> PartialEq for NamedItem<'a, T>
where
	T: Clone + Debug + PartialEq + Eq + Hash + Serialize + Deserialize<'a>,
{
	fn eq(&self, other: &Self) -> bool {
		(self.name, self.base, self.inner) == (other.name, other.base, self.inner)
	}
}

impl<'a, T> Eq for NamedItem<'a, T> where
	T: Clone + Debug + PartialEq + Eq + Hash + Serialize + Deserialize<'a>
{
}

#[derive(Debug)]
pub enum ItemErrorKind {
	MissingItem,
	MissingName,
	ExpectedType(ItemKind),
	MultipleExports(Id, Id),
}

#[derive(Debug)]
pub struct ItemError {
	id: Id,
	kind: ItemErrorKind,
}

impl ItemError {
	pub fn new(id: Id, kind: ItemErrorKind) -> Self {
		Self {
			id,
			kind,
		}
	}
}

impl fmt::Display for ItemError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match &self.kind {
			ItemErrorKind::MissingItem => write!(f, "could not find item: {}", self.id.0),
			ItemErrorKind::MissingName => write!(f, "missing name for item: {}", self.id.0),
			ItemErrorKind::ExpectedType(kind) => {
				write!(f, "expected type {:?} for item: {}", kind, self.id.0)
			}
			ItemErrorKind::MultipleExports(rhs, lhs) => {
				write!(
					f,
					"multiple exports (example {} and {}) for item: {}",
					rhs.0, lhs.0, self.id.0
				)
			}
		}
	}
}

pub fn get<'a>(doc: &'a Crate, id: &'a Id) -> Result<&'a Item, ItemError> {
	let Some(item) = doc.index.get(id) else {
		return Err(ItemError {
			id: doc.root.clone(),
			kind: ItemErrorKind::MissingItem,
		});
	};
	Ok(item)
}

pub fn get_mut<'a>(doc: &'a mut Crate, id: &Id) -> Result<&'a mut Item, ItemError> {
	let Some(item) = doc.index.get_mut(id) else {
		return Err(ItemError {
			id: doc.root.clone(),
			kind: ItemErrorKind::MissingItem,
		});
	};
	Ok(item)
}

pub fn root_module(doc: &Crate) -> Result<NamedItem<Module>, ItemError> {
	let root_module = get(doc, &doc.root)?;
	match &root_module.inner {
		ItemEnum::Module(module) => {
			if let Some(name) = &root_module.name {
				return Ok(NamedItem {
					name,
					base: root_module,
					inner: module,
				});
			}
			Err(ItemError {
				id: doc.root.clone(),
				kind: ItemErrorKind::MissingName,
			})
		}
		_ => {
			Err(ItemError {
				id: doc.root.clone(),
				kind: ItemErrorKind::ExpectedType(ItemKind::Module),
			})
		}
	}
}

struct Parent<'a> {
	id: &'a Id,
	alias: Option<&'a String>,
}

pub struct PathResolver<'a> {
	doc: &'a Crate,
	root_module: NamedItem<'a, Module>,
	child_parent_map: HashMap<&'a Id, &'a Id>,
	import_map: HashMap<&'a Id, Vec<Parent<'a>>>,
}

impl<'a> PathResolver<'a> {
	pub fn from(doc: &'a Crate) -> Result<Self, ItemError> {
		let root_module = root_module(doc)?;

		let mut child_parent_map = HashMap::new();
		let mut import_map = HashMap::<&'a Id, Vec<Parent<'a>>>::new();
		let mut queue = vec![(&root_module.base.id, root_module.inner)];
		while let Some((module_id, module)) = queue.pop() {
			for child_id in &module.items {
				if let Some(reexport) = child_parent_map.insert(child_id, module_id) {
					return Err(ItemError {
						id: child_id.clone(),
						kind: ItemErrorKind::MultipleExports(module_id.clone(), reexport.clone()),
					});
				}
				let Some(child_item) = doc.index.get(child_id) else {
					return Err(ItemError {
						id: child_id.clone(),
						kind: ItemErrorKind::MissingItem,
					});
				};
				match &child_item.inner {
					ItemEnum::Module(child_module) => {
						queue.push((child_id, child_module));
					}
					ItemEnum::Use(use_item) => {
						if let Some(import_id) = &use_item.id {
							import_map.entry(import_id).or_default().push(Parent {
								id: module_id,
								alias: Some(&use_item.name),
							});
						}
					}
					_ => {}
				}
			}
		}
		Ok(Self {
			doc,
			root_module,
			child_parent_map,
			import_map,
		})
	}

	pub fn canonical_parent(&self, module_id: &Id) -> Option<&Id> {
		self.child_parent_map.get(module_id).copied().or_else(|| {
			if let Some(parent_list) = self.import_map.get(module_id) {
				if parent_list.len() == 1 {
					return Some(parent_list[0].id);
				} else {
					return self.shortest_import(parent_list).0;
				}
			}
			return None;
		})
	}

	fn shortest_parent(&self, item_id: &Id) -> (Option<&Id>, usize) {
		if let Some(parent) = self.child_parent_map.get(item_id) {
			return (Some(parent), self.shortest_parent(parent).1);
		}
		if let Some(parent_list) = self.import_map.get(item_id) {
			if parent_list.len() == 1 {
				let parent = parent_list[0].id;
				return (Some(parent), self.shortest_parent(parent).1);
			}
			return self.shortest_import(parent_list);
		}
		(None, 0)
	}

	fn shortest_import(&self, parent_list: &Vec<Parent<'a>>) -> (Option<&Id>, usize) {
		if parent_list.len() == 1 {
			let parent = parent_list[0].id;
			return (Some(parent), self.shortest_parent(parent).1);
		}
		let mut shortest_parent = None;
		let mut shortest_len = usize::MAX;
		for parent in parent_list {
			let path = self.shortest_parent(parent.id);
			if path.1 < shortest_len {
				shortest_parent = path.0;
				shortest_len = path.1;
			}
		}
		return (shortest_parent, shortest_len);
	}

	pub fn root(&self) -> &NamedItem<'a, Module> {
		&self.root_module
	}

	pub fn doc(&self) -> &Crate {
		self.doc
	}
}

pub fn find_item<'a>(doc: &'a Crate, name: &[&str]) -> Option<&'a Id> {
	if name.is_empty() {
		return None;
	}
	for (id, item_summary) in &doc.paths {
		if item_summary.path == name {
			return Some(id);
		}
	}
	None
}
