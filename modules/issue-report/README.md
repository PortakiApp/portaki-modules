# issue-report

Official Portaki issue report — guests report problems during the stay without WhatsApp.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`issue-report`

OCI image: `ghcr.io/portakiapp/portaki-modules-issue-report:<semver>`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | `IssueReport` entity (many per stay) |

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Category + summary form; list of this stay’s reports after submit |
| host | `main` | Module info + recent reports (up to 20), « Marquer comme résolu » on open ones |
| host | `issue-stats` | `property-stats-card` — reports, resolved / open, resolution time and categories over `input.periodDays` (30 / 90 / 365) |

## Queries and commands

- `listForStay` — reports for the current guest stay
- `listRecent` — newest reports for the property (host)
- `submit` — create report; emits `issue-report.submitted`
- `resolve` — host marks a report resolved (`resolved_at`, first call wins)

## Development

```bash
cargo test -p issue-report
cd modules/issue-report
portaki build --release
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
