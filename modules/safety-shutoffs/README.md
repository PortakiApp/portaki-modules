# safety-shutoffs

Official Portaki safety shutoffs module — where to cut the power, the water and the gas, and where
the extinguisher and smoke detectors are.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`safety-shutoffs`

OCI image: `oci.portaki.app/modules/safety-shutoffs:<semver>`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | Host configuration (`shutoffs`, `general_note`) |

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | A discreet card in the Help section — no content, opens the detail |
| guest | `explore.detail` | Fullscreen: the 112 banner, the host note, one block per shutoff |
| host | `main` | Up to eight dynamic rows + the general instruction |

## Queries and commands

- `publishReadiness` — refuses to publish without one complete shutoff point
- `updateConfig` — the platform persists the host form itself

## Design notes

- The home card carries **no** danger tone and lists nothing: a guest arriving should not read
  about the gas valve first (§2.22).
- The detail holds the booklet's single danger block, and its text is fixed — a gas smell is not
  answered by closing a valve, it is answered by leaving and calling 112. No `EmergencyButton`:
  the red register stays with 112.
- The call button reads the host's phone from `context.host`, never from a field of this module.
  The runtime does not serve `context.host` yet, so today the button does not render — which is
  what the SDK prescribes for `HostProfile::phone`.
- Photos wait for the file-storage lot: `core.images` has no host API in the SDK.

## Development

```bash
cargo test -p safety-shutoffs
cd modules/safety-shutoffs
portaki build --release
```

## Verify the artifact

Every published version is attested by this repository's `ci` workflow on `main` (keyless cosign,
Rekor transparency log), with SLSA provenance and the `cargo audit` report attached. A Portaki
production runtime only runs a signed digest. To check one yourself:

```bash
cosign verify-attestation --type slsaprovenance1 oci.portaki.app/modules/safety-shutoffs@<digest> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/PortakiApp/portaki-modules/.github/workflows/ci.yml@refs/heads/main
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
