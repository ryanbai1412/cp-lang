# CI, packaging, distribution and versioning

Status: proposed, for the morning the repo goes public. Supersedes the
release parts of `plan/b0-distribution.md`.

## What ships

| Artifact | Built from | Where it goes |
|---|---|---|
| `cpl` binary: compiler, interpreter, LSP, fmt, lint, doc, stress, judge, DAP; stdlib embedded | `crates/cpl-cli` | GitHub Releases |
| Targets | `x86_64`/`aarch64-unknown-linux-musl` (static), `x86_64`/`aarch64-apple-darwin`, `x86_64-pc-windows-msvc` | one archive each + `.sha256` |
| `cpl.wasm` + JS worker | `crates/cpl-wasm` | baked into the site build; also a release asset |
| Site (docs, Learn, playground) | `web/` static export `web/out` | Cloudflare Pages, `cp-lang.org` |
| Short-link Worker (`/p/<hash>`, R2) | `web/worker` | Cloudflare Workers, route `cp-lang.org/p/*` |
| `install.sh` / `install.ps1` | generated at release | `cp-lang.org/install.sh`, plus release assets |
| Editor packages | `editors/*` | see "Editors" below |

One binary is one version: the stdlib ships inside it, so a version number
names the compiler, the stdlib and the judge profiles together. g++ is only
needed for `cpl build`/`judge` runs that compile; `cpl doctor` says what is
missing.

## Versioning

- Single source: `[workspace.package] version` in `Cargo.toml`. Bump it to
  `0.3.0` when v0.3 ships (today it says `0.2.2`).
- Pre-1.0 semver: a minor bump (`0.3` → `0.4`) may break code and must ship
  `cpl fix --upgrade` rewrites; a patch bump never breaks valid programs.
- Three channels:

  | Channel | What | Install |
  |---|---|---|
  | stable | tagged releases (`v0.3.0`) | `curl -fsSL https://cp-lang.org/install.sh \| sh` |
  | edge | every green `main` commit, prebuilt, one CI run (~15 min) behind `main` | `... \| sh -s -- --edge`; `cpl self update` stays on edge |
  | source | any commit, even red | `cargo install --locked --git https://github.com/ryanbai1412/cp-lang cpl-cli [--rev <sha>]` |

  Edge builds are versioned `0.3.0-edge.<n>+<sha>` and published as a
  rolling `edge` GitHub pre-release. Bisecting uses source builds
  (`--rev <sha>`); there is no per-commit binary install. `cpl --version` prints channel, version and commit.
- Publishing cost: edge is one rolling release whose assets are replaced
  (5 archives of about 10 MB, plus checksums). `release.yml` uses a
  `concurrency` group with cancel-in-progress, so bursts of pushes give
  one build (at most about 4 per hour). One dated prerelease per day (`edge-YYYY-MM-DD`)
  is kept permanently.
- Archives: stable tags and daily edge prereleases are permanent; anything
  else rebuilds from source at its commit. Every build records its commit
  SHA in `cpl --version`, the release notes, `latest.json` and the stamp
  on every generated C++ file, so `cargo install --git ... --rev <sha>`
  always reproduces it. Two channels for now; a separate daily `nightly`
  channel is one manifest entry if edge ever moves too fast.
- A bad edge build: `cpl self rollback` (`self update` keeps the previous
  toolchain locally, so it works offline), `cpl +stable ...`, `cpl self
  install edge-YYYY-MM-DD`, or a source build at any SHA, then `cpl report`.
- `cpl --version` prints version, commit and build date. Every generated C++
  file carries a stamp (`// cpl 0.3.0 (abc1234)`) so old submissions can be
  rebuilt with the same compiler.
- Compatibility until 1.0: edge-style, no guarantees, three rules.
  Every release lists its breaking changes in the changelog. A renamed API
  keeps a compile error with a `cpl fix` rewrite for one minor version
  (from the spec's rename table). Old solutions rebuild with the version in
  their stamp (`CPL_VERSION=0.3.0`). Editions and real stability guarantees
  start at 1.0.
- Builds are reproducible: `--locked`, pinned Rust via `mise.toml` /
  `rust-toolchain.toml`, `SOURCE_DATE_EPOCH` from the commit.

## Install

- `curl -fsSL https://cp-lang.org/install.sh | sh` (Windows:
  `irm https://cp-lang.org/install.ps1 | iex`). It detects the platform,
  downloads the archive from the GitHub release, checks its sha256, installs
  to `~/.cpl/bin` without root and prints the PATH line. `CPL_VERSION=0.3.0`
  or `--edge` picks a version or channel.
- Version management works like rustup. `~/.cpl/toolchains/<version>/`
  holds each installed build, and `~/.cpl/bin/cpl` is a small launcher that
  picks one. Commands: `cpl self channel stable|edge` sets the global
  default; `cpl self update` follows the channel; `cpl self install
  <version>`; `cpl self list`. A project pins with `version = "0.3.0"` or
  `channel = "edge"` in `cpl.toml`, and `cpl +edge run a.cp` is a one-off.
  Precedence: `+x`, then `CPL_VERSION`, then `cpl.toml`, then the global
  default. `cpl --version` names the build and why it was chosen. All of
  these read one release manifest (`cp-lang.org/releases/latest.json`,
  written at release time). Launch needs only the channel and `self
  update`; pins and `+x` come after.
- Also supported: `cargo install --locked --git https://github.com/ryanbai1412/cp-lang cpl-cli`.
- Later: Homebrew tap, Scoop bucket.
- Tool: [`dist`] (cargo-dist)
  generates the release workflow, archives, checksums and the shell and
  PowerShell installers. We host its installer at `cp-lang.org/install.sh`
  instead of hand-writing one.

## Editors

Every editor runs the same `cpl lsp`, so a release only packages the
editor-side pieces. Each package finds `cpl` on PATH and otherwise says how to
install it. The VS Code and JetBrains packages can also download the matching
release. Every package is versioned with the compiler and built by
`release.yml`, and each package is also a release asset.

No marketplace publishing for now: every release attaches the editor
packages as assets, and /setup gives a manual install line per editor.

| Editor | Release asset | Install line on /setup |
|---|---|---|
| VS Code (and Cursor, Windsurf, VSCodium) | `cp-lang.vsix` from `editors/vscode` | `code --install-extension cp-lang.vsix` (or Extensions, "Install from VSIX") |
| JetBrains IDEs | `cp-lang-jetbrains.zip` from `editors/jetbrains` | Settings, Plugins, gear, "Install Plugin from Disk" |
| Zed | `editors.tar.gz` (`zed/`) | Extensions, "Install Dev Extension", pick the folder |
| Neovim/Vim | `editors.tar.gz` (`neovim/`) | add the folder to `runtimepath` (or point lazy.nvim at it with `dir =`) |
| Helix | `editors.tar.gz` (`helix/languages.toml`) | paste into `~/.config/helix/languages.toml`, then `hx --grammar fetch` and `hx --grammar build` |
| Emacs | `editors.tar.gz` (`emacs/cp-mode.el`) | `(load "/path/to/cp-mode.el")`; eglot config documented |
| tree-sitter grammar | `editors.tar.gz` (`tree-sitter-cp/`, with prebuilt WASM) | used by Zed, Helix and Neovim |

- **CI:** `full.yml` installs each editor (VS Code headless, Neovim, Emacs,
  Helix, Zed's CLI, JDK for JetBrains) and runs `editors/smoke.sh`, which
  starts `cpl lsp` the way each config does and checks `initialize` plus a
  published error. It also builds the grammar and the JetBrains plugin. A
  failing editor blocks the release. Today only VS Code has real tests.
- **Later:** VS Code Marketplace and Open VSX, then Zed, JetBrains, MELPA,
  grammar and Neovim mirror repos, once the language settles.

## Bug reports

- `cpl report a.cp < in.txt` writes `cpl-report-<hash>.md`, ready to paste
  into an issue. It holds the version, channel and commit, the OS, the
  `g++` version and judge profile, the source, the input (cut down by the
  stress shrinker with `--shrink`), stdout, stderr and exit codes for both
  the interpreter and the C++ build, whether they disagree, and any
  internal-error backtrace.
- An internal compiler error prints "this is a cpl bug: run `cpl report`"
  and saves the crash to `~/.cpl/crashes/`, so `cpl report --last` works
  afterwards.
- A small report also prints a prefilled GitHub new-issue URL;
  `.github/ISSUE_TEMPLATE/bug.yml` asks for the report.
- Privacy: no environment variables and no absolute paths. The command
  prints what the file contains before you share it.

## CI (GitHub Actions, free once public)

Four workflows. Every job runs `mise install` first, so CI and laptops use
the same tool versions and the same `just` recipes; no logic lives only in
YAML.

1. **`ci.yml`**: on PRs and pushes to `main`. `just check` (today's
   `scripts/check.sh --fast`) on Ubuntu, with rust-cache and the pnpm cache;
   cancel superseded runs. A smaller `cargo test -p cpl-cli` on macOS and
   Windows so platform breaks show before a release. Required for merging.
2. **`full.yml`**: nightly and on release tags. The full gate: WASM build and
   smoke, VS Code tests, editor smoke, prover fuzzing, latency budgets, the
   `cpl-conform` scoreboard (conformance, corpus, doc examples) with its
   ratchet, and interpreter-vs-C++ differential under each judge profile's
   compiler and flags. A failing ratchet fails the job.
3. **`release.yml`**: on `v*` tags (stable), and after `ci.yml` passes on `main` (edge). Build
   the target matrix, checksums, GitHub artifact attestations (signed build
   provenance), the installers and every editor package; publish the GitHub
   Release; publish the editor packages and grammar to their registries;
   push the mirrors; write `releases/latest.json` for the site.
4. **`site.yml`**: on `main` changes to `web/`, `spec/`, `std`/`stdlib` or
   the crates the WASM build uses. Build `cpl.wasm`, run `pnpm build`
   (which regenerates the reference from `cpl doc --format json`), run the
   site gate (every snippet runs or is marked checks-only), then
   `wrangler pages deploy web/out` and `wrangler deploy` for the Worker.
   PRs get Pages preview URLs. Secrets: `CLOUDFLARE_API_TOKEN`,
   `CLOUDFLARE_ACCOUNT_ID`.

With required checks and GitHub's merge queue, `scripts/land.sh` and its
lock refs can be deleted: that is the queue they emulate.

## Before the repo goes public

1. **Licence:** MIT OR Apache-2.0 (decided). Add `LICENSE-MIT` and
   `LICENSE-APACHE`, and set `license` in `Cargo.toml`.
2. **Corpus:** problem statements plus Ryan's and Andrew's code stay public
   (decided).
3. **History:** nothing sensitive anywhere in history: secrets, internal
   hosts, session links, home paths, and anything about Cognition or its
   environments, including commit author names and emails. Scan everything
   (gitleaks plus a pattern scan of every blob and commit message), then
   either publish a fresh repo with one clean commit (recommended) or
   rewrite history with `git filter-repo` (needs a force-push). Ryan
   decides.
4. `README.md` with the install line and playground link; `CONTRIBUTING.md`
   pointing at `AGENTS.md` and `just check`; `SECURITY.md`.

## Morning order

1. Pick fresh history or filter-repo from the scan report; I prepare it.
2. Make the repo public.
3. Add the Cloudflare secrets; create the Pages project `cp-lang` and an R2
   bucket for short links; point `cp-lang.org` at Pages.
4. Merge the workflows PR (I'll have it ready: `ci.yml`, `full.yml`,
   `release.yml`, `site.yml`, `dist` config).
5. Tag `v0.3.0`, or ship edge only if v0.3 isn't green yet. CI builds,
   releases and deploys; `curl cp-lang.org/install.sh | sh` works.
