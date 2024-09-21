use std::env;
use std::fs;
use std::io::{self};
use std::path::{self};
use std::process::Command;
use std::string::FromUtf8Error;

use log::error;
use log::info;
use log::warn;
use thiserror::Error;

fn main() -> anyhow::Result<()> {
	env_logger::init();

	let rust_src_dir = std::env::var("RUST_SRC_DIR").unwrap_or("./rust".to_owned());
	let rust_src_path = path::absolute(&rust_src_dir)?.canonicalize()?;
	if rust_src_path.exists() {
		info!("Using rust source directory: {}", rust_src_path.display());
	// TODO: Fetch latest changes.
	} else {
		if let Some(rust_src_parent_dir) = rust_src_path.parent() {
			info!("Creating rust source directory: {}", rust_src_parent_dir.display());
			fs::create_dir_all(rust_src_parent_dir)?;
		}
		error!("Please clone the Rust source code. See the README for detailed instructions")
		// TODO: Clone.
	}

	// TODO: Rebuild changes (optionally).

	let target_env = std::env::var("CARGO_BUILD_TARGET").or_else(|_| std::env::var("TARGET"));
	let target_path = target_env
		.ok()
		.and_then(|target| {
			let std_rustdoc_path = rust_src_path.join(rustdoc_build_path(&target));
			if std_rustdoc_path.exists() {
				info!("Using target: {}", target);
				Some(std_rustdoc_path)
			} else {
				warn!("Invalid target: {}", target);
				None
			}
		})
		.or_else(|| {
			info!("Looking up possible targets...");
			// See also: https://github.com/rust-lang/cargo/issues/3946
			let rustc_dir = std::env::var("CARGO_RUSTC_CURRENT_DIR").ok();
			let targets = targets(rustc_dir.as_deref()).unwrap();
			targets
				.iter()
				.filter(|target| is_native_target(target))
				.map(|target| (target, rust_src_path.join(rustdoc_build_path(target))))
				.find(|(_, std_rustdoc_path)| std_rustdoc_path.exists())
				.map(|(target, std_rustdoc_path)| {
					info!("Detected existing target: {}", target);
					std_rustdoc_path
				})
		})
		.unwrap();

	fs::copy(target_path, "./data/std.json")?;
	info!("Done!");
	Ok(())
}

// Returns the list of targets that can be built for this OS and architecture.
fn targets(rustc_dir: Option<&str>) -> Result<Vec<String>, CommandError> {
	let mut command = Command::new("rustc");
	command.args([
		"--print",
		"target-list",
	]);
	if let Some(dir) = rustc_dir {
		command.current_dir(dir);
	}
	let output = command.output()?;
	if !output.status.success() {
		return Err(CommandError::ExitCode(output.status.code()));
	}
	let contents = String::from_utf8(output.stdout);
	match contents {
		Ok(targets) => Ok(targets.split('\n').map(str::to_owned).collect()),
		Err(e) => Err(CommandError::ParseError(e)),
	}
}

#[derive(Error, Debug)]
pub enum CommandError {
	#[error("unable to run command")]
	Io(#[from] io::Error),
	#[error("unsuccessful exit (code {0:?})")]
	ExitCode(Option<i32>),
	#[error("unable to parse output")]
	ParseError(#[from] FromUtf8Error),
}

/// Returns true if the given target matches the current architecture and OS.
fn is_native_target(target: &str) -> bool {
	let arch = env::consts::ARCH;
	target.starts_with(arch) && target[arch.len()..].contains(env::consts::OS)
}

fn rustdoc_build_path(target: &str) -> String {
	format!("build/{}/doc/std.json", target)
}
