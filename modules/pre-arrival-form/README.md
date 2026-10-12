# pre-arrival-form

Official Portaki pre-arrival form — ETA, occasion, allergies, and a message to the host.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`pre-arrival-form`

OCI image: `oci.portaki.app/modules/pre-arrival-form:<semver>`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | `PreArrivalResponse` entity (unique per stay) |

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Inline form when available; thank-you when done; hidden when gated |
| host | `main` | Config editor (`show_when` + question toggles) |
| host | `stay` | Stay-detail embed — read-only responses (`input.stayId`) |
| host | `resend` | Stay action « Renvoyer le formulaire » — channel, e-mail preview, `resendForm` |
| host | `forms` | Stats tile « Formulaires remplis » (`statsSummary`, count over the period) |

Host entries (`#[surface]` placements): `property-workspace-tab`, `stay-detail` (`stay`), `stay-action` (`resend`), `property-stats-card` (`forms`).

## Queries and commands

- `getStatus` — `{ completed, arrivalTimeEstimated?, … }` for the current stay
- `submit` — upsert response; emits `pre-arrival.completed`
- `updateConfig` — persist `show_when` + question flags
- `sendFormAvailable` — daily tick + manifest catch-up (`emails[].catchUpOnPropertyPublish` / `catchUpOnConfigUpdate` / `dispatchOnStayCreated`); `host::email::send` when form is available (dedup `form-available`)
- `resendForm` — host, `{ stayId }`: the `form-available` e-mail again, as `form-resend-<uuid>`; refused once answered or after check-in
- `statsSummary` — forms completed over the period

## Development

```bash
cargo test -p pre-arrival-form
cd modules/pre-arrival-form
portaki build --release
```

## Verify the artifact

Every published version is attested by this repository's `ci` workflow on `main` (keyless cosign,
Rekor transparency log), with SLSA provenance and the `cargo audit` report attached. A Portaki
production runtime only runs a signed digest. To check one yourself:

```bash
cosign verify-attestation --type slsaprovenance1 oci.portaki.app/modules/pre-arrival-form@<digest> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/PortakiApp/portaki-modules/.github/workflows/ci.yml@refs/heads/main
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
