## Summary

<!-- What changed and why (1–3 bullets). -->

-

## Test plan

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] For touched modules: `portaki build --release && portaki lint`
- [ ] Version bump in `modules/<id>/Cargo.toml` if publishing a new image
