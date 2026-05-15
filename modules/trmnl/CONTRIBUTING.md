# Contributing

Issues and pull requests are welcome on [PortakiApp/portaki-modules](https://github.com/PortakiApp/portaki-modules) (path `modules/trmnl/`).

- Keep payloads small (under 2kb for free TRMNL tier).
- Do not add tenant identifiers, JWTs, or full guest email addresses to `merge_variables`.
- Prefer snake_case keys shared between Java (`TrmnlPayloadBuilder`) and Liquid templates.
