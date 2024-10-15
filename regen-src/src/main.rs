#![warn(clippy::pedantic)]
mod print;

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
									function_list.push(NamedItem {
										name,
										item,
										inner: function,
									});
								}
							}
							rustdoc_types::ItemEnum::Struct(doc_struct) => {
								if let Some(name) = &item.name {
									struct_list.push(NamedItem {
										name,
										item,
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

struct NamedItem<'a, T: Eq> {
	name: &'a String,
	item: &'a rustdoc_types::Item,
	inner: &'a T,
}

impl<'a, T: Eq> Ord for NamedItem<'a, T> {
	fn cmp(&self, other: &Self) -> Ordering {
		let name_ordering = self.name.cmp(other.name);
		if name_ordering == Ordering::Equal {
			self.item.id.0.cmp(&other.item.id.0)
		} else {
			name_ordering
		}
	}
}

impl<'a, T: Eq> PartialOrd for NamedItem<'a, T> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a, T: Eq> PartialEq for NamedItem<'a, T> {
	fn eq(&self, other: &Self) -> bool {
		(self.name, self.item, self.inner) == (other.name, other.item, self.inner)
	}
}

impl<'a, T: Eq> Eq for NamedItem<'a, T> {}

fn generate_structs(
	output_path: impl AsRef<Path>,
	buf: &mut Vec<u8>,
	doc_crate: &rustdoc_types::Crate,
	struct_list: &Vec<NamedItem<rustdoc_types::Struct>>,
) -> io::Result<()> {
	info!("Generating structs.rs...");
	for item in struct_list {
		writeln!(buf)?;
		print::write_doc(buf, item.item)?;
		writeln!(buf, "pub trait {} {{", item.name)?;
		for impl_id in &item.inner.impls {
			if let Some(impl_item) = doc_crate.index.get(impl_id) {
				if let ItemEnum::Impl(doc_impl) = &impl_item.inner {
					if let Some(impl_trait) = &doc_impl.trait_ {
						write!(buf, "// impl ")?;
						print::write_path(buf, doc_crate, impl_trait)?;
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
	function_list: &Vec<NamedItem<rustdoc_types::Function>>,
) -> io::Result<()> {
	info!("Generating functions.rs...");
	writeln!(buf)?;
	writeln!(buf, "pub trait Fs {{")?;
	for item in function_list {
		writeln!(buf)?;
		print::write_doc(buf, item.item)?;
		print::write_function(buf, doc_crate, item.name, item.inner)?;
		writeln!(buf, ";")?;
	}
	writeln!(buf, "}}")?;

	writeln!(buf)?;
	writeln!(buf, "pub struct Native {{}}")?;
	writeln!(buf)?;
	writeln!(buf, "impl Fs for Native {{")?;
	for item in function_list {
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
