> [!NOTE]  
> This library is a WIP and not functional yet!

## Regenerating

This library generates an `fs` wrapper by running `rustdoc` on the standard library itself, using the `--output-format json` option.

There are two helper scripts:
1. `regen-doc`: Builds and copies the rust-lang `rustdoc` output into `data/std.json`. It only needs to be run whenever there is an update in the standard library.
2. `regen-src`: Processes `data/std.json` and output valid Rust source code into `src/generated.rs`.

### regen-doc

First, clone this repository. Then, navigate to it. Next, run the script. Run in your shell of choice:

```bash
RUST_LOG=info cargo run --package regen-doc
```

If you already have the Rust source installed on your system and would rather use that, run:

```bash
RUST_LOG=info RUST_SRC_DIR=path/to/rust REGEN_RUST_SRC_CONF_SKIP=1 cargo run --package regen-doc
```

#### Environment Variables

The regeneration script supports the following environment variables:

Name                       | Default   | Description
---------------------------|-----------|------------
`CARGO_BUILD_TARGET`       |           | The build target to use. If unset, attempts to discover the target that the Rust standard library was built with.
`CARGO_RUSTC_CURRENT_DIR`  |           | The `rustc` directory to use when discovering available targets. If unset, uses the globally installed `rustc` instance.
`REGEN_RUSTDOC_SKIP`       | `0`       | Set to `1` to disable skipping rebuilding the Rustdoc.
`REGEN_RUST_SRC_PULL_SKIP` | `0`       | Set to `1` to disable skipping updating the Rust source.
`REGEN_RUST_SRC_CONF_SKIP` | `0`       | Set to `1` to disable skipping overriding the default Rust source configuration file.
`RUSTDOCFLAGS`             |           | Additional flags to build Rustdocs with.
`RUST_SRC_DIR`             | `./rust/` | The directory to the Rust installation.
`RUST_SRC_REF`             | `master`  | The ref to build the Rust source from.
`RUST_SRC_REMOTE`          | `origin`  | The remote to pull Resource source upstream from.
`TARGET`                   |           | Alias for `CARGO_BUILD_TARGET`.

### regen-src

Run:

```bash
RUST_LOG=info cargo run --package regen-src
```
