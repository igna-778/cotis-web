# Security Policy

## Supported versions

Security fixes are provided for the latest release on the `main` branch and the most recent tagged release. cotis-web is in early `0.1.0-alpha` development; upgrade to the latest version when possible.

| Version | Supported |
|---------|-----------|
| `main` (latest) | Yes |
| Latest tagged release | Yes |
| Older pre-release tags | Best effort |

This policy applies to the publishable crates in this repository:

- `cotis-web`
- `cotis-web-builder`

Vulnerabilities in ecosystem crates ([cotis](https://github.com/igna-778/cotis), [cotis-cli](https://github.com/igna-778/cotis-cli), [cotis-layout](https://github.com/igna-778/cotis-layout), etc.) should be reported to the respective repository maintainers.

## Reporting a vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, use one of the following private channels:

1. **GitHub Security Advisories (preferred)**  
   Open a [private vulnerability report](https://github.com/igna-778/cotis-web/security/advisories/new) on this repository.

2. **Direct contact**  
   If GitHub advisories are not available, contact the maintainer through the contact information on their GitHub profile ([@igna-778](https://github.com/igna-778)).

### What to include

To help us assess and fix the issue quickly, please provide:

- A description of the vulnerability and its potential impact
- Steps to reproduce, or a minimal proof of concept if applicable
- Affected crate(s), version(s), and feature flags if relevant
- Any suggested mitigation or fix, if you have one

### What to expect

- **Acknowledgement** within 7 days of a valid report
- **Status updates** as the issue is triaged and addressed
- **Coordinated disclosure** — we ask that you do not disclose the issue publicly until a fix is available, unless otherwise agreed
- **Credit** in the release notes or advisory, if you would like to be credited

## Scope

The following are generally **in scope**:

- Memory safety issues in Rust code shipped in this repository
- Logic flaws that could lead to denial of service in library consumers
- Unsafe code blocks and their soundness guarantees
- Dependency vulnerabilities introduced or significantly affected by this project
- Issues in the build routine that could lead to arbitrary code execution during builds

The following are generally **out of scope**:

- Issues in downstream applications that misuse the public API
- Vulnerabilities in third-party dependencies already tracked upstream (please report to the upstream project; we will update dependencies as needed)
- Theoretical issues without a practical exploit or impact demonstration
- Social engineering or physical attacks

## Safe harbor

We support good-faith security research. We will not pursue legal action against researchers who:

- Make a good-faith effort to avoid privacy violations, data destruction, and service disruption
- Report vulnerabilities privately and allow reasonable time for a fix before public disclosure
- Do not exploit vulnerabilities beyond what is necessary to demonstrate the issue

Thank you for helping keep cotis-web and its users safe.
