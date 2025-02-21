# CODA Tools

Some tools for working with CODA test instruments (Internal to ESS).

## Build

### Requirements

- A rust toolchain.
- CMake
- cross:

```shell
cargo install cross
```

### Compile

This builds an executable that can run on the DMSC cluster:

```shell
cross build --verbose --release --target x86_64-unknown-linux-gnu
```

Note that this does *not* run on login nodes because they have a too old version of gcc.

## Run

The binary is called `codai`.
Use `codai -h` to get help on command line arguments.
