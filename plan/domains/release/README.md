# release

Segment: A. Service. Owns: `.github/workflows`, `scripts/release`, installers, public-repo hygiene. Worklog: [worklog.md](worklog.md).

## Covers
CI workflows, release pipeline, installers, versioning, the public-repo surface (CONTRIBUTING, SECURITY, license, history hygiene).

## Improve
First edge + nightly; install smoke on 3 OSes; CI on push/PR; commit-message + pushed-range scan in CI; `full.yml`; version 0.3.0; fresh public repo when Ryan says.

## Success criteria
`curl -fsSL https://cp-lang.org/install.sh | sh` installs a working `cpl` on Linux/macOS/Windows from stable and edge; nightly edge 7 days in a row; `v0.3.0` cut.

## Ryan's role
Approves cutting `v0.3.0` and the fresh public repo; otherwise none.
