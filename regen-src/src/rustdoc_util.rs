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
}

#[derive(Debug)]
pub struct ItemError {
	id: Id,
	kind: ItemErrorKind,
}

impl fmt::Display for ItemError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match &self.kind {
			ItemErrorKind::MissingItem => write!(f, "could not find item: {}", self.id.0),
			ItemErrorKind::MissingName => write!(f, "missing name for item: {}", self.id.0),
			ItemErrorKind::ExpectedType(kind) => {
				write!(f, "expected type {:?} for item: {}", kind, self.id.0)
			}
		}
	}
}

pub fn root_module(root: &Crate) -> Result<NamedItem<Module>, ItemError> {
	let Some(root_module) = root.index.get(&root.root) else {
		return Err(ItemError {
			id: root.root.clone(),
			kind: ItemErrorKind::MissingItem,
		});
	};
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
				id: root.root.clone(),
				kind: ItemErrorKind::MissingName,
			})
		}
		_ => {
			Err(ItemError {
				id: root.root.clone(),
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
	child_parent_alias_map: HashMap<&'a Id, Vec<Parent<'a>>>,
}

impl<'a> PathResolver<'a> {
	pub fn from(doc: &'a Crate) -> Result<Self, ItemError> {
		let root_module = root_module(doc)?;

		let mut child_parent_map = HashMap::new();
		let mut child_parent_alias_map = HashMap::<&'a Id, Vec<Parent<'a>>>::new();
		let mut queue = vec![(&root_module.base.id, root_module.inner)];
		while let Some((module_id, module)) = queue.pop() {
			for child_id in &module.items {
				child_parent_map.insert(child_id, module_id);
				child_parent_alias_map.entry(child_id).or_default().push(Parent {
					id: module_id,
					alias: None,
				});
				let Some(child_item) = doc.index.get(child_id) else {
					return Err(ItemError {
						id: child_id.clone(),
						kind: ItemErrorKind::MissingItem,
					});
				};
				let mut child_item = child_item;
				loop {
					match &child_item.inner {
						ItemEnum::Module(child_module) => {
							child_parent_map.insert(child_id, module_id);
							child_parent_alias_map.entry(child_id).or_default().push(Parent {
								id: module_id,
								alias: None,
							});
							queue.push((child_id, child_module));
						}
						ItemEnum::Import {
							id: Some(import_id),
							name,
							glob,
							..
						} => {
							let alias = if *glob {
								None
							} else {
								Some(name)
							};
							child_parent_alias_map.entry(import_id).or_default().push(Parent {
								id: module_id,
								alias,
							});

							// Built-in types will not be found.
							if let Some(import_item) = doc.index.get(import_id) {
								child_item = import_item;
								continue;
							}
						}
						_ => {}
					}
					break;
				}
			}
		}
		Ok(Self {
			doc,
			root_module,
			child_parent_map,
			child_parent_alias_map,
		})
	}

	pub fn parent(&self, module_id: &Id) -> Option<&Id> {
		self.child_parent_map.get(module_id).copied()
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
