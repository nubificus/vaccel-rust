# Rust bindings and RPC components for vAccel

Crates implementing Rust interfaces for the vAccel C library:
- The `vaccel-bindings` crate provides Rust bindings for vAccel
- The `vaccel-rpc-proto`, `vaccel-rpc-agent` and `vaccel-rpc-client` crates
  implement RPC-based transport for vAccel operations. They can be used
  directly, to interact with vAccel from Rust, or through the C interface, to
  implement transport plugins for vAccel.

## Install vAccel

Install vAccel with:
```bash
wget https://s3.nbfc.io/nbfc-assets/github/vaccel/rev/main/x86_64/release/vaccel_latest_x86_64.deb
sudo dpkg -i vaccel_latest_x86_64.deb
```

## Build the components

First clone the repo:
```bash
git clone git@github.com:nubificus/vaccel-rust
cd vaccel-rust
```

### Build with Cargo

All Rust components can be used as dependencies and be built with Cargo as
usual. This repo is structured as a Cargo workspace so all crates can be
compiled from the root directory.

```bash
cargo build
```
will build the `vaccel-rpc-agent` (and dependencies)

Crates can also be built separately, ie.:
```bash
cargo build -p vaccel-bindings
```
will build  `vaccel-bindings` (and dependencies).

### Build with Meson

A Meson build is also provided for ease of integration with vAccel. The Meson
implementation uses Cargo for the actual build and provides build targets for
`vaccel-rpc-agent` and `vaccel-rpc-client`. To build with Meson:
```bash
meson setup build
meson compile -C build
```

This will build `vaccel-rpc-agent` (and dependencies)  using sync
[ttrpc](https://github.com/containerd/ttrpc-rust).

To also build `vaccel-rpc-client` use:
```bash
meson setup -Drpc-client=enabled build
meson compile -C build
```

There is experimental support for a Rust RPC agent/client based on async
ttrpc and a streaming implementation of the vAccel `genop` operation. To
compile with these features you can configure the project with:
```bash
# async ttrpc
meson setup -Dasync=enabled build

# async ttrpc with streaming genop
meson setup -Dasync-stream=enabled build
```
