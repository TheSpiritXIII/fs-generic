#![warn(clippy::pedantic)]
use std::any::Any;
use std::env;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::{self};
use std::panic::panic_any;
use std::path::Path;
use std::path::PathBuf;
use std::path::{self};
use std::process::Command;
use std::process::Stdio;
use std::string::FromUtf8Error;
use std::thread;

use log::error;
use log::info;
use log::warn;
use serde::ser::Serialize;
use thiserror::Error;

const CONFIG_FILE: &str = include_str!("../data/config.toml");

fn main() -> anyhow::Result<()> {
	env_logger::init();

	let rust_src_dir = std::env::var("RUST_SRC_DIR").unwrap_or("./rust".to_owned());
	let rust_src_path = path::absolute(&rust_src_dir)?;
	let should_skip_rustdoc = match std::env::var("REGEN_RUSTDOC_SKIP") {
		Ok(should_skip_env) => {
			if let Some(is_truthy) = is_truthy(&should_skip_env) {
				is_truthy
			} else {
				error!("Invalid REGEN_RUSTDOC_SKIP value: {}", should_skip_env);
				std::process::exit(1);
			}
		}
		Err(_) => false,
	};
	if should_skip_rustdoc {
		info!("Skipping regenerating rustdoc.");
	} else {
		info!("Regenerating rustdoc...");

		if rust_src_path.exists() {
			info!("Using rust source directory: {}", rust_src_path.display());

			let skip_pull_env = std::env::var("REGEN_RUST_SRC_PULL_SKIP");
			if skip_pull_env.is_ok_and(|value| is_truthy(&value).unwrap_or(false)) {
				info!("Skipping pulling latest Rust source.");
			} else {
				let has_changes = git_has_changes(&rust_src_path)?;
				if has_changes {
					error!("Rust source directory has changes. Please commit or stash them.");
					std::process::exit(1);
				}

				let rust_src_remote =
					std::env::var("RUST_SRC_REMOTE").unwrap_or("origin".to_owned());
				let rust_src_ref = std::env::var("RUST_SRC_REF").unwrap_or("master".to_owned());
				info!("Switching to {}/{}...", &rust_src_remote, &rust_src_ref);
				git_switch(&rust_src_dir, &rust_src_ref)?;
				info!("Pulling latest changes...");
				git_pull(&rust_src_dir, &rust_src_remote, &rust_src_ref)?;

				let has_changes = git_has_changes(&rust_src_path)?;
				if has_changes {
					error!("Rust source directory has changes. Please check for merge conflicts.");
					std::process::exit(1);
				}

				let skip_pull_env = std::env::var("REGEN_RUST_SRC_CONF_SKIP");
				if skip_pull_env.is_ok_and(|value| is_truthy(&value).unwrap_or(false)) {
					info!("Skipping updating config.toml.");
				} else {
					info!("Updating config.toml file...");
					fs::write(rust_src_path.join("config.toml"), CONFIG_FILE)?;
				}
			}
		} else if let Some(rust_src_parent_dir) = rust_src_path.parent() {
			info!("Creating rust source directory: {}", rust_src_parent_dir.display());
			fs::create_dir_all(rust_src_parent_dir)?;

			git_clone(rust_src_parent_dir, "https://github.com/rust-lang/rust.git")?;

			info!("Creating default config.toml file...");
			fs::write(rust_src_path.join("config.toml"), CONFIG_FILE)?;
		} else {
			error!("Cannot find parent directory: {}", rust_src_path.display());
			std::process::exit(1);
		}

		regen_rustdoc(&rust_src_path)?;
	}

	let target_path_option = discover_target(rust_src_path);
	let Some(target_path) = target_path_option else {
		error!("Unable to find a build target. Was rustdoc built?");
		std::process::exit(1)
	};

	info!("Copying and formatting data to this project...");
	let data_raw = fs::read_to_string(target_path)?;
	let data = prettify_json(&data_raw)?;
	fs::write("./data/std.json", data)?;

	info!("Done!");
	Ok(())
}

fn prettify_json(raw: &str) -> anyhow::Result<Vec<u8>> {
	let data: serde_json::Value = serde_json::from_str(raw)?;
	let mut buf = Vec::new();
	let formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");
	let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
	data.serialize(&mut ser)?;
	Ok(buf)
}

fn discover_target(rust_src_path: impl AsRef<Path>) -> Option<PathBuf> {
	let target_env = std::env::var("CARGO_BUILD_TARGET").or_else(|_| std::env::var("TARGET"));
	target_env
		.ok()
		.and_then(|target| {
			let std_rustdoc_path = rust_src_path.as_ref().join(rustdoc_build_path(&target));
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
			let targets = match targets(rustc_dir.as_deref()) {
				Ok(targets) => targets,
				Err(e) => {
					error!("Unable to discover rustc targets (is rustc available?): {}", e);
					std::process::exit(1)
				}
			};
			targets
				.iter()
				.filter(|target| is_native_target(target))
				.map(|target| (target, rust_src_path.as_ref().join(rustdoc_build_path(target))))
				.find(|(_, std_rustdoc_path)| std_rustdoc_path.exists())
				.map(|(target, std_rustdoc_path)| {
					info!("Detected existing target: {}", target);
					std_rustdoc_path
				})
		})
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
		return Err(CommandError::ExitCode(output.status.code().unwrap_or(-1)));
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
	#[error("unsuccessful exit (code {0})")]
	ExitCode(i32),
	#[error("unable to parse output")]
	ParseError(#[from] FromUtf8Error),
}

/// Returns true if the given target matches the current architecture and OS.
fn is_native_target(target: &str) -> bool {
	let arch = env::consts::ARCH;
	target.starts_with(arch) && target[arch.len()..].contains(env::consts::OS)
}

fn rustdoc_build_path(target: &str) -> String {
	format!("build/{target}/doc/std.json")
}

fn regen_rustdoc(rust_src_dir: impl AsRef<Path>) -> io::Result<()> {
	let mut command = Command::new("python");
	command.current_dir(rust_src_dir);
	command.args([
		"x.py",
		"doc",
		"library/std",
		// Specify stage explicitly otherwise this fails on GitHub Actions.
		"--stage",
		"0",
	]);
	// Rust-lang refuses to download LLVM in GitHub Actions, so trick it.
	command.env_remove("GITHUB_ACTIONS");
	command.env(
		"RUSTDOCFLAGS",
		format!(
			"{} --output-format json",
			std::env::var("RUSTDOCFLAGS").unwrap_or_else(|_| String::new())
		),
	);
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

fn is_truthy(value: &str) -> Option<bool> {
	let normalized = value.to_lowercase();
	if normalized == "1" || normalized == "true" {
		return Some(true);
	}
	if normalized == "0" || normalized == "false" {
		return Some(false);
	}
	None
}

fn git_has_changes(repo_dir: impl AsRef<Path>) -> io::Result<bool> {
	let mut command = Command::new("git");
	command.current_dir(repo_dir);
	command.args([
		"diff",
		"--quiet",
	]);
	let output = command.output()?;
	Ok(!output.status.success())
}

fn git_switch(repo_dir: impl AsRef<Path>, reference: &str) -> io::Result<()> {
	let mut command = Command::new("git");
	command.current_dir(repo_dir);
	command.args([
		"switch",
		reference,
	]);
	command_redirect_output(command)
}

fn git_pull(repo_dir: impl AsRef<Path>, remote: &str, reference: &str) -> io::Result<()> {
	let mut command = Command::new("git");
	command.current_dir(repo_dir);
	command.args([
		"pull",
		remote,
		reference,
		"--progress",
	]);
	command_redirect_output(command)
}

fn git_clone(repo_dir: impl AsRef<Path>, url: &str) -> io::Result<()> {
	let mut command = Command::new("git");
	command.current_dir(repo_dir);
	command.args([
		"clone",
		url,
		"--depth",
		"1",
	]);
	command_redirect_output(command)
}
