# Tontoo SDK

The Tontoo SDK is the single dependency every TontooOS app needs: thin,
safe Rust bindings for the system frameworks in `/Library/System`. Apps
declare `sdk::frameworks!()` once and then use frameworks like normal
crates, while the actual framework code stays on the system and is loaded
at runtime.

## Feature Index

| Main index | [MAIN.md](MAIN.md) | Entry point: overview + index of all features |
| Rules | [RULE.md](RULE.md) | Wiki authoring rules |
| System bindings | [Sdk.md](Sdk.md) | Dynamic loading, shim macro, per-framework modules |
