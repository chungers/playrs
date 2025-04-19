Playrs
======
Collection of stuff in rust.


Developing
----------
For emacs rust-mode, for example).
After checking out the repo, do

```
git submodule update --init
```

Install `protoc`, from [gRPC docs](https://grpc.io/docs/protoc-installation/)

```
$ brew install protobuf
$ protoc --version  # Ensure compiler version is 3+
```


Useful
------
Download and install MacDown for .md files

```
brew install --cask macdown
```

### Managing Rust dependencies

Install cargo-edit 

```
cargo install cargo-edit
```

Running from project directory:

```
cargo upgrade -i --dry-run
```

Install cargo-upgrades to track latest versions of crates

```
cargo install -f cargo-upgrades
```

Notes
-----
Aug 14, 2024 -- Upgrade `time` crate to fix compile error after upgrading to rustc v1.8
  + Issue: [Type inference regression on nightly-2024-05-2](https://github.com/rust-lang/rust/issues/125319)

```
cargo update -p time
```
