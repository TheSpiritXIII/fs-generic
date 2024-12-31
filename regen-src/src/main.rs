#![warn(clippy::pedantic)]
mod print;
mod rustdoc_util;
mod visitor;

use std::any::Any;
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
use rustdoc_types::Id;
use rustdoc_types::ItemEnum;
use thiserror::Error;

const HEADER: &str = include_str!("../data/header.rs");

fn main() -> anyhow::Result<()> {
	env_logger::init();

	let input_path = path::absolute("data/std.json")?;
	info!("Parsing doc from {}...", input_path.display());
	let input_data = fs::read_to_string(input_path)?;
	let mut doc_crate = serde_json::from_str(&input_data)?;
	// remove_preludes(&mut doc_crate).map_err(SourceError::ParseError)?;

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

fn remove_preludes(doc: &mut rustdoc_types::Crate) -> Result<(), rustdoc_util::ItemError> {
	let mut prelude_index = None;
	{
		let root_module = rustdoc_util::root_module(doc)?;
		for (index, id) in root_module.inner.items.iter().enumerate() {
			if let Some(item) = doc.index.get(id) {
				if let Some(name) = &item.name {
					if name == "prelude" {
						prelude_index = Some((index, id.clone()));
						break;
					}
				}
			}
		}
	}

	let root_item = rustdoc_util::get_mut(doc, &doc.root.clone()).unwrap();
	let rustdoc_types::ItemEnum::Module(root_module) = &mut root_item.inner else {
		unreachable!("already checked type earlier");
	};
	if let Some((index, id)) = prelude_index {
		root_module.items.remove(index);
		doc.index.remove(&id);
	}
	Ok(())
}

#[derive(Error, Debug)]
pub enum SourceError {
	#[error("unable to write source")]
	Io(#[from] io::Error),
	#[error("parse error: {0}")]
	ParseError(rustdoc_util::ItemError),
}

fn json_to_rs(doc: &rustdoc_types::Crate, output_dir: impl AsRef<Path>) -> Result<(), SourceError> {
	let mut buf = Vec::new();

	let path_resolver = rustdoc_util::PathResolver::from(doc).map_err(SourceError::ParseError)?;
	// if let Some(x) = path_resolver.path(&rustdoc_types::Id("0:3782:7948".to_owned())) {
	// 	info!("Out: {}", x.iter().copied().cloned().collect::<Vec<String>>().join("::"));
	// }
	// exit(1);
	let root_module = path_resolver.root();
	for id in &root_module.inner.items {
		if let Some(item) = doc.index.get(id) {
			if let rustdoc_types::ItemEnum::Module(item_module) = &item.inner {
				if item.name.as_ref().is_some_and(|name| name == "fs") {
					let mut function_list = Vec::new();
					let mut struct_list = Vec::new();

					for id in &item_module.items {
						let Some(item) = doc.index.get(id) else {
							continue;
						};
						match &item.inner {
							rustdoc_types::ItemEnum::Function(function) => {
								if let Some(name) = &item.name {
									function_list.push(rustdoc_util::NamedItem {
										name,
										base: item,
										inner: function,
									});
								}
							}
							rustdoc_types::ItemEnum::Struct(doc_struct) => {
								if let Some(name) = &item.name {
									struct_list.push(rustdoc_util::NamedItem {
										name,
										base: item,
										inner: doc_struct,
									});
								}
							}
							_ => {}
						}
					}

					function_list.sort();
					struct_list.sort();

					generate_structs(
						output_dir.as_ref().join("structs.rs"),
						&mut buf,
						&path_resolver,
						&struct_list,
					)?;
					buf.clear();

					generate_functions(
						output_dir.as_ref().join("functions.rs"),
						&mut buf,
						doc,
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
	path_resolver: &rustdoc_util::PathResolver,
	struct_list: &Vec<rustdoc_util::NamedItem<rustdoc_types::Struct>>,
) -> io::Result<()> {
	info!("Generating structs.rs...");
	let doc_crate = path_resolver.doc();
	let deny = rustdoc_util::find_item(
		doc_crate,
		&[
			"core",
			"marker",
			"StructuralPartialEq",
		],
	)
	.unwrap();

	for item in struct_list {
		writeln!(buf)?;
		print::write_doc(buf, item.base)?;
		writeln!(buf, "pub trait {} {{", item.name)?;
		for impl_id in &item.inner.impls {
			if let Some(impl_item) = doc_crate.index.get(impl_id) {
				if let ItemEnum::Impl(doc_impl) = &impl_item.inner {
					if let Some(impl_trait) = &doc_impl.trait_ {
						if &impl_trait.id == deny {
							continue;
						}
						if doc_impl.blanket_impl.is_some() || doc_impl.synthetic {
							continue;
						}

						if !visitor::visit_item(impl_item, &|id| {
							if has_module_with_name(path_resolver, id, "windows") {
								return false;
							}
							if has_module_with_name(path_resolver, id, "unix") {
								return false;
							}
							if has_module_with_name(path_resolver, id, "linux") {
								return false;
							}
							if has_module_with_name(path_resolver, id, "wasi") {
								return false;
							}
							true
						}) {
							continue;
						}

						write!(buf, "// impl ")?;
						print::write_path(buf, &path_resolver, doc_crate, impl_trait)?;
						writeln!(buf)?;
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
								print::write_function_args(buf, doc_crate, impl_func)?;
								writeln!(buf, ";")?;
							}
						}
					}
				}
			}
		}
		writeln!(buf, "}}")?;
		writeln!(buf)?;
		writeln!(buf, "// impl {0} for std::fs::{0} {{}}", item.name)?;
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
	function_list: &Vec<rustdoc_util::NamedItem<rustdoc_types::Function>>,
) -> io::Result<()> {
	info!("Generating functions.rs...");
	writeln!(buf)?;
	writeln!(buf, "pub trait Fs {{")?;
	for item in function_list {
		if item.base.deprecation.is_some() {
			continue;
		}

		writeln!(buf)?;
		print::write_doc(buf, item.base)?;
		print::write_function(buf, doc_crate, item.name, item.inner)?;
		writeln!(buf, ";")?;
	}
	writeln!(buf, "}}")?;

	writeln!(buf)?;
	writeln!(buf, "pub struct Native {{}}")?;
	writeln!(buf)?;
	writeln!(buf, "impl Fs for Native {{")?;
	for item in function_list {
		if item.base.deprecation.is_some() {
			continue;
		}

		writeln!(buf)?;
		print::write_function(buf, doc_crate, item.name, item.inner)?;
		writeln!(buf, " {{")?;
		write!(buf, "	std::fs::{}(", item.name)?;
		for (input_name, _) in &item.inner.decl.inputs {
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
use std::path;
"
	)?;
	out_file.write_all(buf)?;
	Ok(())
}

// Hacky but works for now. Would like to check full path instead.
fn has_module_with_name(path_resolver: &rustdoc_util::PathResolver, id: &Id, name: &str) -> bool {
	let mut id = id;
	while let Some(parent) = path_resolver.canonical_parent(id) {
		id = parent;
		if let Some(item) = path_resolver.doc().index.get(id) {
			if let Some(item_name) = &item.name {
				if item_name == name {
					return true;
				}
			}
		}
	}
	false
}
