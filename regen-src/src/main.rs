#![warn(clippy::pedantic)]
use std::fs::OpenOptions;
use std::io::Write;
use std::path;

use log::info;

const HEADER: &str = include_str!("../data/header.rs");

fn main() -> anyhow::Result<()> {
	env_logger::init();

	let output_path = path::absolute("src/generated.rs")?;
	info!("Regenerating source into {}...", output_path.display());
	let mut out_file =
		OpenOptions::new().write(true).create(true).truncate(true).open(output_path)?;
	write!(out_file, "{HEADER}")?;
	info!("Done!");
	Ok(())
}
