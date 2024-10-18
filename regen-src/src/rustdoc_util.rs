use std::cmp::Ordering;
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
