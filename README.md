# cusf_sidechain

CUSF sidechain node (experimental).

## Build

```bash
git submodule update --init --recursive
cargo build
```

Protos for the in-tree **`bip300301_enforcer_proto`** shim live under **`cusf_sidechain_proto/proto/`** (git submodule).

## Follow-ups (maintainers)

- **Publish:** Push with a GitHub user that can write **LayerTwo-Labs** remotes (`403` means wrong or insufficient credentials).
- **Proto crate pin:** To align **`Cargo.lock`**’s **`cusf_sidechain_proto`** dependency with a newer **`cusf_sidechain_proto`** repo revision, run  
  `cargo update -p cusf_sidechain_proto`  
  and run **`cargo test`** again.
