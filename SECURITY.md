# Security and sensitive data

Patina is early-stage software. Only the current development version is actively
maintained; there is no promise of backported security fixes for older snapshots.

## Reporting a vulnerability

Use GitHub's **Security → Report a vulnerability** for the repository if private
reporting is enabled. If it is unavailable, open an issue asking the maintainer
for a private reporting channel, without including the exploit or sensitive data.
Do not post access tokens, session credentials, account databases or private keys
in a public issue, pull request, screenshot or attachment.

Include the affected component, version/commit, platform, impact and a minimal
reproduction that avoids real credentials and private worlds.

## Safe diagnostics and contributions

- `accounts.json` contains authentication data. Keep it private.
- Remove access/refresh tokens, device codes, session join data, user identifiers
  and private server addresses from logs before sharing them.
- Use the browser preview's synthetic account for screenshots.
- Keep local settings, worlds, game resources, crash reports and backups out of Git.
- Do not add secrets to `.env.example`; use placeholders.
- Verify a download's source and checksum before running a release.

The source-release validator checks known token patterns and private/generated
paths. It is a useful guard, not a proof that arbitrary source or assets are safe
or correctly licensed. Review the selected files before publishing.

The optional client is a separate, Horizon-derived codebase with network,
telemetry and mod-related functionality. Review its settings and code for your
build. Launcher tests do not certify that functionality.
