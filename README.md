## Regenerating

This library generates an `fs` wrapper by running `rustdoc` on the standard library itself, using the `--output-format json` option.

A helper script is included to copy that file into this repository. It only needs to be run whenever there is an update in the standard library.

First, clone this repository. Then, navigate to it. Next, run the script. Run in your shell of choice:

```bash
RUST_LOG=info cargo run --package regen
```

### Environment Variables

The regeneration script supports the following environment variables:

Name                       | Default   | Description
---------------------------|-----------|------------
`RUST_SRC_DIR`             | `./rust/` | The directory to the Rust installation.
`CARGO_BUILD_TARGET`       |           | The first built-target found that matches the current OS and architecture. If unset, discovers the target that the Rust standard library was built with.
`TARGET`                   |           | Alias for `CARGO_BUILD_TARGET`.
`CARGO_RUSTC_CURRENT_DIR`  |           | The `rustc` directory to use when discovering available targets. If unset, uses the globally installed `rustc` instance.
`RUSTDOCFLAGS`             |           | Additional flags to build Rustdocs with.
`REGEN_RUSTDOC_SKIP`       | `0`       | Set to `1` to disable skipping rebuilding the Rustdoc.
`REGEN_RUST_SRC_PULL_SKIP` | `0`       | Set to disable skipping updating the Rust source.
`RUST_SRC_REMOTE`          | `origin`  | The remote to pull from upstream when `REGEN_SKIP_UPDATE` is unset.
`RUST_SRC_REF`             | `master`  | The ref to build the Rust source from.
