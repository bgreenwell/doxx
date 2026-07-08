# Security Policy

## Reporting a Vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities.

Instead, use GitHub's private vulnerability reporting:

1. Go to the [Security tab](https://github.com/bgreenwell/doxx/security) of this repository.
2. Click **"Report a vulnerability"**.
3. Describe the issue, including steps to reproduce and, if applicable, a sample `.docx` file that triggers the problem.

This opens a private conversation with the maintainer, so the issue can be assessed and fixed before it's public.

## What's in scope

doxx parses untrusted `.docx` files (which are ZIP archives of XML), so the areas of most interest are:

- Crashes or panics when parsing a malformed/malicious `.docx` file (denial of service)
- Path traversal or arbitrary file write during image extraction (`--extract-images`, `ImageExtractor`)
- Memory-safety issues (should be prevented by Rust, but `unsafe` misuse or dependency vulnerabilities are still possible)
- Vulnerabilities in direct dependencies that are actually reachable through doxx's usage

## What's likely out of scope

- Issues requiring an already-compromised machine or local file access beyond what doxx itself would need
- Rendering/display bugs with no security impact (those belong in a regular issue)

## Response

This is a small, actively-maintained open source project without a dedicated security team or SLA. Reports will be acknowledged and triaged as soon as reasonably possible. Fixes for confirmed vulnerabilities will be released promptly and noted in `CHANGELOG.md` under a `### Security` entry.
