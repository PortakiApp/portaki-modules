# Module source layout

Canonical Wasm crate layout for crates under `modules/`:

- `ids.rs` — `define_event_types!` and `module_id()`, only when the module needs them
- `guest/` — `#[surface(guest, …)]` only
- `host/` — `#[surface(host, …)]` only (omit for guest-only modules)
- `connectors` / `connectors.rs` — when the module talks to external pools

Typed boundary ids: `#[surface(id = "home.card")]` declares `HOME_CARD` and
`#[query(name = "statsSummary")]` / `#[command]` declare `STATS_SUMMARY` next to the
handler (SDK **8.7.0+**). Use those consts at every `Action::*` / `Surface::with_id` /
`events::emit` call site.

Full rules live in the sibling SDK docs:

- [module-layout.md](../../portaki-sdk/docs/module-layout.md)
- [typed-ids.md](../../portaki-sdk/docs/typed-ids.md)
