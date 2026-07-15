# Security Policy

## Supported versions

Security fixes target the default branch (`main`) and the latest published module versions on GHCR when applicable.

## Reporting a vulnerability

Please **do not** file a public GitHub issue for security vulnerabilities.

Email **security@syntax-labs.fr** with:

- A short description of the issue
- Affected module id / version (and OCI digest if known)
- Steps to reproduce or a proof of concept
- Your preferred contact method for follow-up

We aim to acknowledge reports within **5 business days**.

## Scope (examples)

In scope:

- Unauthorized data access through module storage or host capabilities
- Injection / XSS-like issues in guest surface payloads that the host trusts
- Credential leakage (BYOK keys, pool tokens) in logs or OCI layers
- Supply-chain compromises in published module images

Out of scope:

- Issues solely in [`portaki-sdk`](https://github.com/PortakiApp/portaki-sdk) (report there)
- Issues solely in the Portaki host / orchestrator
- Denial of service requiring already-elevated host access

## Prefer responsible disclosure

Give us a reasonable window to ship a fix before public disclosure. We will coordinate a timeline when needed.
