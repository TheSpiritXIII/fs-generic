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
	T: Fn(Id) -> bool,
{
	if !visitor(item.id) {
		return false;
	}
	match &item.inner {
		ItemEnum::Module(module) => {
			for id in &module.items {
				if !visitor(*id) {
					return false;
				}
			}
		}
		ItemEnum::ExternCrate {
			..
		}
		| ItemEnum::Macro(_) => return true,
		ItemEnum::Use(use_item) => {
			if let Some(id) = &use_item.id {
				if !visitor(*id) {
					return false;
				}
			}
		}
		ItemEnum::Union(union) => {
			for id in &union.fields {
				if !visitor(*id) {
					return false;
				}
			}
			for id in &union.impls {
				if !visitor(*id) {
					return false;
				}
			}
			unimplemented!();
		}
		ItemEnum::Struct(_) => unimplemented!(),
		ItemEnum::StructField(_) => unimplemented!(),
		ItemEnum::Enum(_) => unimplemented!(),
		ItemEnum::Variant(_) => unimplemented!(),
		ItemEnum::Function(_) => unimplemented!(),
		ItemEnum::Trait(_) => unimplemented!(),
		ItemEnum::TraitAlias(_) => unimplemented!(),
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
				if !visit_path(path, visitor) {
					return false;
				}
			}
			for id in &impl_item.items {
				if !visitor(*id) {
					return false;
				}
			}
			return visit_generic_params(&impl_item.generics.params, visitor);
		}
		ItemEnum::TypeAlias(_) => unimplemented!(),
		ItemEnum::Constant {
			type_,
			..
		}
		| ItemEnum::AssocConst {
			type_,
			..
		} => return visit_type(type_, visitor),
		ItemEnum::Static(_) => unimplemented!(),
		ItemEnum::ExternType => unimplemented!(),
		ItemEnum::ProcMacro(_) => unimplemented!(),
		ItemEnum::Primitive(_) => unimplemented!(),
		ItemEnum::AssocType {
			..
		} => unimplemented!(),
	}
	true
}

fn visit_path<T>(path: &Path, visitor: &T) -> bool
where
	T: Fn(Id) -> bool,
{
	if !visitor(path.id) {
		return false;
	}
	if let Some(args) = &path.args {
		if !visit_generic_args(args, visitor) {
			return false;
		}
	}
	true
}

fn visit_type<T>(item_type: &Type, visitor: &T) -> bool
where
	T: Fn(Id) -> bool,
{
	match item_type {
		Type::ResolvedPath(path) => visit_path(path, visitor),
		Type::DynTrait(dyn_trait) => {
			for poly_trait in &dyn_trait.traits {
				if !visit_path(&poly_trait.trait_, visitor) {
					return false;
				}
				if !visit_generic_params(&poly_trait.generic_params, visitor) {
					return false;
				}
			}
			true
		}
		Type::Generic(_) | Type::Primitive(_) => true,
		Type::FunctionPointer(_) => unimplemented!("visit function pointer"),
		Type::Tuple(vec) => {
			for tuple_type in vec {
				if !visit_type(tuple_type, visitor) {
					return false;
				}
			}
			true
		}
		Type::Slice(_) => unimplemented!(),
		Type::Array {
			type_,
			..
		}
		| Type::Pat {
			type_,
			..
		}
		| Type::RawPointer {
			type_,
			..
		}
		| Type::BorrowedRef {
			type_,
			..
		} => visit_type(type_, visitor),
		Type::ImplTrait(vec) => visit_generic_bounds(vec, visitor),
		Type::Infer => unimplemented!(),
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
			visit_type(self_type, visitor)
		}
	}
}

fn visit_generic_args<T>(args: &GenericArgs, visitor: &T) -> bool
where
	T: Fn(Id) -> bool,
{
	match args {
		rustdoc_types::GenericArgs::AngleBracketed {
			args,
			constraints,
		} => {
			for arg in args {
				match arg {
					rustdoc_types::GenericArg::Type(arg_type) => {
						if !visit_type(arg_type, visitor) {
							return false;
						}
					}
					rustdoc_types::GenericArg::Lifetime(_)
					| rustdoc_types::GenericArg::Const(_)
					| rustdoc_types::GenericArg::Infer => return true,
				}
			}
			for constraint in constraints {
				if !visit_generic_args(&constraint.args, visitor) {
					return false;
				}
				match &constraint.binding {
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
	true
}

fn visit_generic_params<T>(generic_params: &[GenericParamDef], visitor: &T) -> bool
where
	T: Fn(Id) -> bool,
{
	for generic in generic_params {
		match &generic.kind {
			rustdoc_types::GenericParamDefKind::Lifetime {
				..
			} => continue,
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
				if !visit_generic_bounds(bounds, visitor) {
					return false;
				}
			}
			rustdoc_types::GenericParamDefKind::Const {
				type_,
				..
			} => {
				if !visit_type(type_, visitor) {
					return false;
				}
			}
		}
	}
	true
}

fn visit_generic_bounds<T>(bounds: &[GenericBound], visitor: &T) -> bool
where
	T: Fn(Id) -> bool,
{
	for bound in bounds {
		match bound {
			rustdoc_types::GenericBound::TraitBound {
				trait_,
				generic_params,
				..
			} => {
				if !visitor(trait_.id) {
					return false;
				}
				if !visit_generic_params(generic_params, visitor) {
					return false;
				}
			}
			rustdoc_types::GenericBound::Outlives(_) | rustdoc_types::GenericBound::Use(_) => {
				continue;
			}
		}
	}
	true
}
