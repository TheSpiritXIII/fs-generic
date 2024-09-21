## Regenerating

This library generates an `fs` wrapper by running `rustdoc` on the standard library itself, using the `--output-format json` option.

A helper script is included to copy that file into this repository. It only needs to be run whenever there is an update in the standard library.

First, clone this repository. Then, navigate to it.

Next, install Rust and run the script. Run in your shell of choice:

```bash
pushd ../
git clone https://github.com/rust-lang/rust.git --depth 1
pushd rust
RUSTDOCFLAGS="--output-format json" ./x.py doc library/std
popd # rust
popd # ../
```

Finally, run the script:

```bash
RUST_SRC_DIR=../rust RUST_LOG=info cargo run --package regen
```

### Environment Variables

The regeneration script supports the following environment variables:

Name | Default | Description
-----|---------|------------
`RUST_SRC_DIR` | `./rust/` | The directory to the Rust installation.
`CARGO_BUILD_TARGET` | The first built-target found that matches the current OS and architecture. | The target that the Rust standard library was built with.
`TARGET` | Alias for `CARGO_BUILD_TARGET`.
`CARGO_RUSTC_CURRENT_DIR` | The globally installed `rustc` command directory. | The `rustc` directory to use when finding available targets.
