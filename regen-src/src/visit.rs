use rustdoc_types::GenericArgs;
use rustdoc_types::GenericBound;
use rustdoc_types::GenericParamDef;
use rustdoc_types::Id;
use rustdoc_types::Item;
use rustdoc_types::ItemEnum;
use rustdoc_types::Path;
use rustdoc_types::Type;

pub fn visit_item<T>(item: &Item, visitor: &T) -> bool
where
	T: Fn(&Id) -> bool,
{
	if !visitor(&item.id) {
		return false;
	}
	match &item.inner {
		ItemEnum::Module(module) => unimplemented!(),
		ItemEnum::ExternCrate {
			..
		} => return true,
		ItemEnum::Use(_) => unimplemented!(),
		ItemEnum::Union(union) => unimplemented!(),
		ItemEnum::Struct(_) => unimplemented!(),
		ItemEnum::StructField(_) => unimplemented!(),
		ItemEnum::Enum(_) => unimplemented!(),
		ItemEnum::Variant(variant) => unimplemented!(),
		ItemEnum::Function(function) => unimplemented!(),
		ItemEnum::Trait(_) => unimplemented!(),
		ItemEnum::TraitAlias(trait_alias) => unimplemented!(),
		ItemEnum::Impl(impl_item) => {
			if !visit_type(&impl_item.for_, visitor) {
				return false;
			}
			if let Some(item_type) = &impl_item.blanket_impl {
				if !visit_type(item_type, visitor) {
					return false;
				}
			}
			if let Some(path) = &impl_item.trait_ {
				if !visit_path(&path, visitor) {
					return false;
				}
			}
			for id in &impl_item.items {
				if !visitor(id) {
					return false;
				}
			}
			return visit_generic_params(&impl_item.generics.params, visitor);
		}
		ItemEnum::TypeAlias(type_alias) => unimplemented!(),
		ItemEnum::Constant {
			type_,
			..
		} => return visit_type(type_, visitor),
		ItemEnum::Static(_) => unimplemented!(),
		ItemEnum::ExternType => unimplemented!(),
		ItemEnum::Macro(_) => return true,
		ItemEnum::ProcMacro(proc_macro) => unimplemented!(),
		ItemEnum::Primitive(primitive) => unimplemented!(),
		ItemEnum::AssocConst {
			type_,
			..
		} => return visit_type(type_, visitor),
		ItemEnum::AssocType {
			generics,
			bounds,
			type_,
		} => unimplemented!(),
		ItemEnum::Import {
			source,
			name,
			id,
			glob,
		} => unimplemented!(),
	}
	return true;
}

fn visit_path<T>(path: &Path, visitor: &T) -> bool
where
	T: Fn(&Id) -> bool,
{
	if !visitor(&path.id) {
		return false;
	}
	if let Some(args) = &path.args {
		if !visit_generic_args(args, visitor) {
			return false;
		}
	}
	return true;
}

fn visit_type<T>(item_type: &Type, visitor: &T) -> bool
where
	T: Fn(&Id) -> bool,
{
	match item_type {
		Type::ResolvedPath(path) => return visit_path(path, visitor),
		Type::DynTrait(dyn_trait) => {
			for poly_trait in &dyn_trait.traits {
				if !visit_path(&poly_trait.trait_, visitor) {
					return false;
				}
				unimplemented!("visit generic params");
			}
			return true;
		}
		Type::Generic(_) => return true,
		Type::Primitive(_) => return true,
		Type::FunctionPointer(_) => unimplemented!("visit function pointer"),
		Type::Tuple(vec) => {
			for tuple_type in vec {
				if !visit_type(tuple_type, visitor) {
					return false;
				}
			}
			return true;
		}
		Type::Slice(_) => unimplemented!(),
		Type::Array {
			type_,
			..
		} => {
			return visit_type(type_, visitor);
		}
		Type::Pat {
			type_,
			..
		} => {
			return visit_type(type_, visitor);
		}
		Type::ImplTrait(vec) => {
			return visit_generic_bounds(vec, visitor);
		}
		Type::Infer => unimplemented!(),
		Type::RawPointer {
			type_,
			..
		} => {
			return visit_type(type_, visitor);
		}
		Type::BorrowedRef {
			type_,
			..
		} => {
			return visit_type(type_, visitor);
		}
		Type::QualifiedPath {
			args,
			self_type,
			trait_,
			..
		} => {
			if !visit_generic_args(args, visitor) {
				return false;
			}
			if let Some(path) = trait_ {
				if !visit_path(path, visitor) {
					return false;
				}
			}
			return visit_type(self_type, visitor);
		}
	}
}

fn visit_generic_args<T>(args: &GenericArgs, visitor: &T) -> bool
where
	T: Fn(&Id) -> bool,
{
	match args {
		rustdoc_types::GenericArgs::AngleBracketed {
			args,
			bindings,
		} => {
			for arg in args {
				match arg {
					rustdoc_types::GenericArg::Lifetime(_) => return true,
					rustdoc_types::GenericArg::Type(arg_type) => {
						if !visit_type(arg_type, visitor) {
							return false;
						}
					}
					rustdoc_types::GenericArg::Const(_) => return true,
					rustdoc_types::GenericArg::Infer => return true,
				}
			}
			for binding in bindings {
				if !visit_generic_args(&binding.args, visitor) {
					return false;
				}
				match &binding.binding {
					rustdoc_types::AssocItemConstraintKind::Equality(term) => {
						match term {
							rustdoc_types::Term::Type(term_type) => {
								if !visit_type(term_type, visitor) {
									return false;
								}
							}
							rustdoc_types::Term::Constant(_) => return true,
						}
					}
					rustdoc_types::AssocItemConstraintKind::Constraint(vec) => {
						if !visit_generic_bounds(vec, visitor) {
							return false;
						}
					}
				}
			}
		}
		rustdoc_types::GenericArgs::Parenthesized {
			inputs,
			output,
		} => {
			for input_type in inputs {
				if !visit_type(input_type, visitor) {
					return false;
				}
			}
			if let Some(output_type) = output {
				if !visit_type(output_type, visitor) {
					return false;
				}
			}
		}
	}
	return true;
}

fn visit_generic_params<T>(generic_params: &[GenericParamDef], visitor: &T) -> bool
where
	T: Fn(&Id) -> bool,
{
	for generic in generic_params {
		match &generic.kind {
			rustdoc_types::GenericParamDefKind::Lifetime {
				..
			} => return true,
			rustdoc_types::GenericParamDefKind::Type {
				bounds,
				default,
				..
			} => {
				if let Some(default_type) = &default {
					if !visit_type(default_type, visitor) {
						return false;
					}
				}
				return visit_generic_bounds(bounds, visitor);
			}
			rustdoc_types::GenericParamDefKind::Const {
				type_,
				..
			} => {
				return visit_type(type_, visitor);
			}
		}
	}
	return true;
}

fn visit_generic_bounds<T>(bounds: &[GenericBound], visitor: &T) -> bool
where
	T: Fn(&Id) -> bool,
{
	for bound in bounds {
		match bound {
			rustdoc_types::GenericBound::TraitBound {
				trait_,
				generic_params,
				..
			} => {
				if !visitor(&trait_.id) {
					return false;
				}
				return visit_generic_params(&generic_params, visitor);
			}
			rustdoc_types::GenericBound::Outlives(_) => return true,
			rustdoc_types::GenericBound::Use(_) => return true,
		}
	}
	return true;
}
