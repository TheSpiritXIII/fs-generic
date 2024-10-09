#![warn(clippy::pedantic)]
use std::cmp::Ordering;
use std::fs;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::io::{self};
use std::path::Path;
use std::path::{self};
use std::process::Command;
use std::process::Stdio;
use std::thread;

use log::info;
use thiserror::Error;

const HEADER: &str = include_str!("../data/header.rs");

fn main() -> anyhow::Result<()> {
	env_logger::init();

	let input_path = path::absolute("data/std.json")?;
	info!("Parsing doc from {}...", input_path.display());
	let input_data = fs::read_to_string(input_path)?;
	let doc_crate = serde_json::from_str(&input_data)?;

	let output_path = path::absolute("src/generated.rs")?;
	info!("Regenerating source into {}...", output_path.display());
	let mut buf = Vec::new();
	write!(buf, "{HEADER}")?;
	json_to_rs(&doc_crate, &mut buf)?;

	let mut out_file =
		OpenOptions::new().write(true).create(true).truncate(true).open(&output_path)?;
	out_file.write_all(&buf)?;

	info!("Formatting generated files...");
	rustfmt(&output_path)?;

	info!("Done!");
	Ok(())
}

fn rustfmt(file: impl AsRef<Path>) -> io::Result<()> {
	let cargo_path = env!("CARGO");
	let manifest_path = env!("CARGO_MANIFEST_DIR");
	let mut command = Command::new(cargo_path);
	command.current_dir(manifest_path);
	command.args([
		"fmt",
		"--",
		file.as_ref().as_os_str().to_str().unwrap(),
	]);
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
		handle.join().expect("join handle")?;
		Ok(())
	})?;
	child.wait()?;
	Ok(())
}

#[derive(Error, Debug)]
pub enum SourceError {
	#[error("unable to write source")]
	Io(#[from] io::Error),
	#[error("parse error: {0}")]
	ParseError(&'static str),
}

fn json_to_rs<W: Write>(doc_crate: &rustdoc_types::Crate, out: &mut W) -> Result<(), SourceError> {
	let Some(root_module) = doc_crate.index.get(&doc_crate.root) else {
		return Err(SourceError::ParseError("could not find root item in index"));
	};
	let rustdoc_types::ItemEnum::Module(doc_module) = &root_module.inner else {
		return Err(SourceError::ParseError("expected root to be a module"));
	};
	for id in &doc_module.items {
		if let Some(item) = doc_crate.index.get(id) {
			if let rustdoc_types::ItemEnum::Module(item_module) = &item.inner {
				if item.name.as_ref().is_some_and(|name| name == "fs") {
					let mut function_list = Vec::new();
					for id in &item_module.items {
						let Some(item) = doc_crate.index.get(id) else {
							continue;
						};
						if let rustdoc_types::ItemEnum::Function(function) = &item.inner {
							if let Some(name) = &item.name {
								function_list.push((name, item, function));
							}
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

					writeln!(out)?;
					writeln!(out, "pub trait Fs {{")?;
					for (_, item, function) in &function_list {
						writeln!(out)?;
						print_doc(out, item)?;
						print_function_definition(out, doc_crate, item, function)?;
						writeln!(out, ";")?;
					}
					writeln!(out, "}}")?;

					writeln!(out)?;
					writeln!(out, "pub struct Native {{}}")?;
					writeln!(out)?;
					writeln!(out, "impl Fs for Native {{")?;
					for (_, item, function) in &function_list {
						writeln!(out)?;
						print_function_wrapper(out, doc_crate, item, function, "std::fs::")?;
					}
					writeln!(out, "}}")?;
				}
			}
		}
	}
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

fn print_function_definition<W: Write>(
	out: &mut W,
	doc_crate: &rustdoc_types::Crate,
	item: &rustdoc_types::Item,
	function: &rustdoc_types::Function,
) -> io::Result<()> {
	write!(out, "fn {}", item.name.as_ref().unwrap())?;
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

fn print_function_wrapper<W: Write>(
	out: &mut W,
	doc_crate: &rustdoc_types::Crate,
	item: &rustdoc_types::Item,
	function: &rustdoc_types::Function,
	prefix: &str,
) -> io::Result<()> {
	print_function_definition(out, doc_crate, item, function)?;

	writeln!(out, " {{")?;
	write!(out, "	{prefix}{}(", item.name.as_ref().unwrap())?;
	for (input_name, _) in &function.decl.inputs {
		write!(out, "{input_name}, ")?;
	}
	writeln!(out, ")")?;
	writeln!(out, "}}")?;
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
		_ => unimplemented!(),
	}
	Ok(())
}
