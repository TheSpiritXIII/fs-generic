use std::io;
use std::io::Write;

use rustdoc_types::Crate;
use rustdoc_types::Function;
use rustdoc_types::GenericArg;
use rustdoc_types::GenericArgs;
use rustdoc_types::GenericBound;
use rustdoc_types::GenericParamDefKind;
use rustdoc_types::Item;
use rustdoc_types::Path;
use rustdoc_types::TraitBoundModifier;
use rustdoc_types::Type;

pub fn write_doc<W: Write>(out: &mut W, item: &Item) -> io::Result<()> {
	if let Some(docs) = &item.docs {
		for line in docs.lines() {
			writeln!(out, "/// {line}")?;
		}
	}
	Ok(())
}

pub fn write_path<W: Write>(out: &mut W, root: &Crate, path: &Path) -> io::Result<()> {
	if let Some(item_summary) = root.paths.get(&path.id) {
		for i in 0..item_summary.path.len() {
			let path = &item_summary.path[i];
			write!(out, "{path}")?;
			if i != item_summary.path.len() - 1 {
				write!(out, "::")?;
			}
		}
	} else {
		write!(out, "{}", path.name)?;
	}
	if let Some(args) = &path.args {
		write_generic_args(out, root, args)?;
	}
	Ok(())
}

pub fn write_resolved_path<W: Write>(out: &mut W, root: &Crate, path: &Path) -> io::Result<()> {
	const CRATE_PATH: &str = "crate::";
	let name = &path.name;
	if let Some(name) = name.strip_prefix(CRATE_PATH) {
		write!(out, "std::{name}")?;
	} else {
		write!(out, "{name}")?;
	}
	if let Some(args) = &path.args {
		write_generic_args(out, root, args)?;
	}
	Ok(())
}

pub fn write_type<W: Write>(out: &mut W, root: &Crate, item_type: &Type) -> io::Result<()> {
	match item_type {
		Type::ResolvedPath(path) => {
			write_resolved_path(out, root, path)?;
		}
		Type::Generic(doc_generic) => {
			write!(out, "{doc_generic}")?;
		}
		Type::Primitive(doc_primitive) => {
			write!(out, "{doc_primitive}")?;
		}
		Type::Tuple(doc_tuple) => {
			write!(out, "(")?;
			for doc_tuple in doc_tuple {
				write_type(out, root, doc_tuple)?;
			}
			write!(out, ")")?;
		}
		Type::Slice(doc_slice) => {
			write!(out, "[")?;
			write_type(out, root, doc_slice)?;
			write!(out, "]")?;
		}
		Type::BorrowedRef {
			lifetime,
			mutable,
			type_,
		} => {
			if lifetime.is_some() {
				unimplemented!();
			}
			write!(out, "&")?;
			if *mutable {
				write!(out, "mut ")?;
			}
			write_type(out, root, type_)?;
		}
		_ => unimplemented!("{item_type:?}"),
	}
	Ok(())
}

pub fn write_generic_args<W: Write>(
	out: &mut W,
	root: &Crate,
	args: &GenericArgs,
) -> io::Result<()> {
	if let GenericArgs::AngleBracketed {
		args,
		bindings,
	} = args
	{
		if !args.is_empty() {
			write!(out, "<")?;
			for arg in args {
				match arg {
					GenericArg::Type(generic_type) => {
						write_type(out, root, generic_type)?;
					}
					_ => unimplemented!(),
				}
				write!(out, ",")?;
			}
			write!(out, ">")?;
		}
		if !bindings.is_empty() {
			unimplemented!();
		}
	} else {
		unimplemented!()
	}
	Ok(())
}

pub fn write_function_args<W: Write>(
	out: &mut W,
	root: &Crate,
	function: &Function,
) -> io::Result<()> {
	if !function.generics.params.is_empty() {
		write!(out, "<")?;
		for generic_param in &function.generics.params {
			write!(out, "{}: ", generic_param.name)?;
			match &generic_param.kind {
				GenericParamDefKind::Type {
					bounds,
					default,
					synthetic,
				} => {
					for bound in bounds {
						match bound {
							GenericBound::TraitBound {
								trait_,
								generic_params,
								modifier,
							} => {
								if !generic_params.is_empty()
									|| *modifier != TraitBoundModifier::None
								{
									unimplemented!();
								}
								write!(out, "{}", trait_.name)?;
								if let Some(args) = &trait_.args {
									write_generic_args(out, root, args)?;
								}
								write!(out, " + ")?;
							}
							_ => todo!(),
						}
					}
					if *synthetic || default.is_some() {
						unimplemented!();
					}
				}
				_ => unimplemented!(),
			}
			// rustdoc_pretty_type(out, &doc_crate, &generic_param.kind)?;
			write!(out, ", ")?;
		}
		write!(out, ">")?;
	}

	write!(out, "(")?;
	for (input_name, input_type) in &function.decl.inputs {
		write!(out, "{input_name}: ")?;
		write_type(out, root, input_type)?;
		write!(out, ", ")?;
	}
	write!(out, ")")?;

	if let Some(output_type) = &function.decl.output {
		write!(out, " -> ")?;
		write_type(out, root, output_type)?;
	}
	Ok(())
}

pub fn write_function<W: Write>(
	out: &mut W,
	root: &Crate,
	name: &str,
	function: &Function,
) -> io::Result<()> {
	write!(out, "fn {name}")?;
	write_function_args(out, root, function)
}