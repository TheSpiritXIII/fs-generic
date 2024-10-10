#![warn(clippy::pedantic)]
use std::any::Any;
use std::cmp::Ordering;
use std::fs;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::io::{self};
use std::panic::panic_any;
use std::path::Path;
use std::path::{self};
use std::process::Command;
use std::process::Stdio;
use std::thread;

use log::info;
use rustdoc_types::ItemEnum;
use thiserror::Error;

const HEADER: &str = include_str!("../data/header.rs");

fn main() -> anyhow::Result<()> {
	env_logger::init();

	let input_path = path::absolute("data/std.json")?;
	info!("Parsing doc from {}...", input_path.display());
	let input_data = fs::read_to_string(input_path)?;
	let doc_crate = serde_json::from_str(&input_data)?;

	let output_dir = path::absolute("src/generated/")?;
	info!("Regenerating source into {}...", output_dir.display());
	json_to_rs(&doc_crate, &output_dir)?;

	info!("Formatting generated files...");
	let paths = fs::read_dir(&output_dir)?
		.map(|entry| entry.map(|entry| entry.path().as_os_str().to_string_lossy().into_owned()))
		.collect::<Result<Vec<_>, _>>()?;
	rustfmt(&paths)?;

	info!("Done!");
	Ok(())
}

fn rustfmt(paths: &[String]) -> io::Result<()> {
	let cargo_path = env!("CARGO");
	let manifest_path = env!("CARGO_MANIFEST_DIR");
	let mut command = Command::new(cargo_path);
	command.current_dir(manifest_path);
	command.args([
		"fmt",
		"--",
	]);
	command.args(paths);
	command_redirect_output(command)
}

fn command_redirect_output(mut command: Command) -> io::Result<()> {
	command.stdout(Stdio::piped()).stderr(Stdio::piped());

	let mut child = command.spawn()?;
	thread::scope::<_, io::Result<()>>(|scope| {
		let handle = scope.spawn::<_, io::Result<()>>(|| {
			if let Some(stdout) = &mut child.stderr {
				let lines = BufReader::new(stdout).lines();
				for line in lines {
					eprintln!("{}", line?);
				}
			}
			Ok(())
		});
		if let Some(stdout) = &mut child.stdout {
			let lines = BufReader::new(stdout).lines();
			for line in lines {
				println!("{}", line?);
			}
		}
		propagate_panic(handle.join())?;
		Ok(())
	})?;
	child.wait()?;
	Ok(())
}

fn propagate_panic<T>(handle_result: Result<T, Box<dyn Any + Send>>) -> T {
	match handle_result {
		Ok(result) => result,
		Err(a) => panic_any(a),
	}
}

#[derive(Error, Debug)]
pub enum SourceError {
	#[error("unable to write source")]
	Io(#[from] io::Error),
	#[error("parse error: {0}")]
	ParseError(&'static str),
}

fn doc_root_module(
	doc_crate: &rustdoc_types::Crate,
) -> Result<&rustdoc_types::Module, &'static str> {
	let Some(root_module) = doc_crate.index.get(&doc_crate.root) else {
		return Err("could not find root item in index");
	};
	match &root_module.inner {
		rustdoc_types::ItemEnum::Module(doc_module) => Ok(doc_module),
		_ => Err("expected root to be a module"),
	}
}

fn json_to_rs(
	doc_crate: &rustdoc_types::Crate,
	output_dir: impl AsRef<Path>,
) -> Result<(), SourceError> {
	let mut buf = Vec::new();
	let doc_module = doc_root_module(doc_crate).map_err(SourceError::ParseError)?;
	for id in &doc_module.items {
		if let Some(item) = doc_crate.index.get(id) {
			if let rustdoc_types::ItemEnum::Module(item_module) = &item.inner {
				if item.name.as_ref().is_some_and(|name| name == "fs") {
					let mut function_list = Vec::new();
					let mut struct_list = Vec::new();

					for id in &item_module.items {
						let Some(item) = doc_crate.index.get(id) else {
							continue;
						};
						match &item.inner {
							rustdoc_types::ItemEnum::Function(function) => {
								if let Some(name) = &item.name {
									function_list.push((name, item, function));
								}
							}
							rustdoc_types::ItemEnum::Struct(doc_struct) => {
								if let Some(name) = &item.name {
									struct_list.push((name, item, doc_struct));
								}
							}
							_ => {}
						}
					}

					function_list.sort_by(|lhs, rhs| -> Ordering {
						let name_ordering = lhs.0.cmp(rhs.0);
						if name_ordering == Ordering::Equal {
							lhs.1.id.0.cmp(&rhs.1.id.0)
						} else {
							name_ordering
						}
					});
					struct_list.sort_by(|lhs, rhs| -> Ordering {
						let name_ordering = lhs.0.cmp(rhs.0);
						if name_ordering == Ordering::Equal {
							lhs.1.id.0.cmp(&rhs.1.id.0)
						} else {
							name_ordering
						}
					});

					generate_structs(
						output_dir.as_ref().join("structs.rs"),
						&mut buf,
						doc_crate,
						&struct_list,
					)?;
					buf.clear();

					generate_functions(
						output_dir.as_ref().join("functions.rs"),
						&mut buf,
						doc_crate,
						&function_list,
					)?;
				}
			}
		}
	}
	Ok(())
}

fn generate_structs(
	output_path: impl AsRef<Path>,
	buf: &mut Vec<u8>,
	doc_crate: &rustdoc_types::Crate,
	struct_list: &Vec<(&String, &rustdoc_types::Item, &rustdoc_types::Struct)>,
) -> io::Result<()> {
	info!("Generating structs.rs...");
	for (name, item, doc_struct) in struct_list {
		writeln!(buf)?;
		print_doc(buf, item)?;
		writeln!(buf, "pub trait {name} {{")?;
		for impl_id in &doc_struct.impls {
			if let Some(impl_item) = doc_crate.index.get(impl_id) {
				if let ItemEnum::Impl(doc_impl) = &impl_item.inner {
					if doc_impl.trait_.is_some() {
						continue;
					}
					for item_id in &doc_impl.items {
						if let Some(impl_item) = doc_crate.index.get(item_id) {
							if let ItemEnum::Function(impl_func) = &impl_item.inner {
								write!(
									buf,
									"// fn {}",
									impl_item.name.as_ref().unwrap_or(&"unknown".to_owned())
								)?;
								print_function_args(buf, doc_crate, impl_func)?;
								writeln!(buf, ";")?;
							}
						}
					}
				}
			}
		}
		writeln!(buf, "}}")?;
		writeln!(buf)?;
		writeln!(buf, "// impl {name} for std::fs::{name} {{}}")?;
	}
	let mut out_file =
		OpenOptions::new().write(true).create(true).truncate(true).open(&output_path)?;
	write!(out_file, "{HEADER}")?;
	out_file.write_all(buf)?;
	Ok(())
}

fn generate_functions(
	output_path: impl AsRef<Path>,
	buf: &mut Vec<u8>,
	doc_crate: &rustdoc_types::Crate,
	function_list: &Vec<(&String, &rustdoc_types::Item, &rustdoc_types::Function)>,
) -> io::Result<()> {
	info!("Generating functions.rs...");
	writeln!(buf)?;
	writeln!(buf, "pub trait Fs {{")?;
	for (name, item, function) in function_list {
		writeln!(buf)?;
		print_doc(buf, item)?;
		write!(buf, "fn {name}")?;
		print_function_args(buf, doc_crate, function)?;
		writeln!(buf, ";")?;
	}
	writeln!(buf, "}}")?;

	writeln!(buf)?;
	writeln!(buf, "pub struct Native {{}}")?;
	writeln!(buf)?;
	writeln!(buf, "impl Fs for Native {{")?;
	for (name, _, function) in function_list {
		writeln!(buf)?;
		write!(buf, "fn {name}")?;
		print_function_args(buf, doc_crate, function)?;
		writeln!(buf, " {{")?;
		write!(buf, "	std::fs::{name}(")?;
		for (input_name, _) in &function.decl.inputs {
			write!(buf, "{input_name}, ")?;
		}
		writeln!(buf, ")")?;
		writeln!(buf, "}}")?;
	}
	writeln!(buf, "}}")?;

	let mut out_file =
		OpenOptions::new().write(true).create(true).truncate(true).open(&output_path)?;
	write!(out_file, "{HEADER}")?;
	write!(
		out_file,
		"use std::io;
use std::fs::Metadata;
use std::fs::ReadDir;
use std::fs::Permissions;
"
	)?;
	out_file.write_all(buf)?;
	Ok(())
}

fn print_doc<W: Write>(out: &mut W, item: &rustdoc_types::Item) -> io::Result<()> {
	if let Some(docs) = &item.docs {
		for line in docs.lines() {
			writeln!(out, "/// {line}")?;
		}
	}
	Ok(())
}

fn print_function_args<W: Write>(
	out: &mut W,
	doc_crate: &rustdoc_types::Crate,
	function: &rustdoc_types::Function,
) -> io::Result<()> {
	if !function.generics.params.is_empty() {
		write!(out, "<")?;
		for generic_param in &function.generics.params {
			write!(out, "{}: ", generic_param.name)?;
			match &generic_param.kind {
				rustdoc_types::GenericParamDefKind::Type {
					bounds,
					default,
					synthetic,
				} => {
					for bound in bounds {
						match bound {
							rustdoc_types::GenericBound::TraitBound {
								trait_,
								generic_params,
								modifier,
							} => {
								if !generic_params.is_empty()
									|| *modifier != rustdoc_types::TraitBoundModifier::None
								{
									unimplemented!();
								}
								write!(out, "{}", trait_.name)?;
								if let Some(args) = &trait_.args {
									print_generic_arg(out, doc_crate, args)?;
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
		print_type(out, doc_crate, input_type)?;
		write!(out, ", ")?;
	}
	write!(out, ")")?;

	if let Some(output_type) = &function.decl.output {
		write!(out, " -> ")?;
		print_type(out, doc_crate, output_type)?;
	}
	Ok(())
}

fn print_generic_arg<W: Write>(
	out: &mut W,
	doc_crate: &rustdoc_types::Crate,
	arg: &rustdoc_types::GenericArgs,
) -> io::Result<()> {
	if let rustdoc_types::GenericArgs::AngleBracketed {
		args,
		bindings,
	} = arg
	{
		write!(out, "<")?;
		for arg in args {
			match arg {
				rustdoc_types::GenericArg::Type(generic_type) => {
					print_type(out, doc_crate, generic_type)?;
				}
				_ => unimplemented!(),
			}
			write!(out, ",")?;
		}
		write!(out, ">")?;
		if !bindings.is_empty() {
			unimplemented!();
		}
	} else {
		unimplemented!()
	}
	Ok(())
}

fn print_type<W: Write>(
	out: &mut W,
	rustdoc_crate: &rustdoc_types::Crate,
	rustdoc_type: &rustdoc_types::Type,
) -> io::Result<()> {
	match rustdoc_type {
		rustdoc_types::Type::ResolvedPath(path) => {
			const CRATE_PATH: &str = "crate::";
			if path.name.starts_with(CRATE_PATH) {
				write!(out, "std::{}", &path.name[CRATE_PATH.len()..])?;
			} else {
				write!(out, "{}", path.name)?;
			}
			if let Some(args) = &path.args {
				print_generic_arg(out, rustdoc_crate, args)?;
			}
		}
		rustdoc_types::Type::Generic(doc_generic) => {
			write!(out, "{doc_generic}")?;
		}
		rustdoc_types::Type::Primitive(doc_primitive) => {
			write!(out, "{doc_primitive}")?;
		}
		rustdoc_types::Type::Tuple(doc_tuple) => {
			write!(out, "(")?;
			for doc_tuple in doc_tuple {
				print_type(out, rustdoc_crate, doc_tuple)?;
			}
			write!(out, ")")?;
		}
		rustdoc_types::Type::Slice(doc_slice) => {
			write!(out, "[")?;
			print_type(out, rustdoc_crate, doc_slice)?;
			write!(out, "]")?;
		}
		rustdoc_types::Type::BorrowedRef {
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
			print_type(out, rustdoc_crate, type_)?;
		}
		_ => unimplemented!("{rustdoc_type:?}"),
	}
	Ok(())
}
