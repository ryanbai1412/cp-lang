# 11. Distribution, versioning, stability

Status: proposed.

## 11.1 Installing

- One static binary per platform (Linux x86-64 and ARM, macOS, Windows),
  containing the compiler, interpreter, language server, formatter and
  linter. The stdlib ships inside it, so a version is one file.
- Installed with a one-line script, or a package manager (Homebrew, a
  `.deb`, Scoop). The script is idempotent and installs into the user's
  home directory without root.
- g++ (or clang++) is needed only for native builds and `cp bench`; debug
  runs don't need it. `cp doctor` checks the toolchain and the judge
  profiles' compilers and says what is missing.
- The VS Code extension downloads the matching binary on first use, or uses
  the one on `PATH`.

### The command name

The CLI is called `cp` throughout the plan, but `cp` is also the Unix copy
command, so the installed binary can't simply be named `cp`: scripts and
shells would get the wrong program. The binary needs another name (to be
decided, for example `cpl`), and the installer offers an optional shell
alias for interactive use only. The plan keeps writing `cp` for
readability.

## 11.2 Versions

- Semantic versions.
- Before 1.0, any release may break code, but every break comes with an
  automatic fix: `cp fix --upgrade` rewrites old source to the new
  version.
- After 1.0, breaks happen only in **editions** (as in Rust), declared per
  project in `cp.toml`; old editions keep compiling forever.

## 11.3 Version stamps and reproducibility

The user asked for this explicitly: old code on the laptop must stay
buildable.

- Every generated C++ file and every build record holds the compiler
  version, the stdlib version and the hashes of the `lib/` snapshot used
  (doc 4, section 5).
- `cp build --as-built old.cpp` reads the stamp and rebuilds with the same
  compiler and library snapshot, fetching the old compiler version through
  the version manager if needed.
- Library snapshots are content-addressed in the local store, so a later
  incompatible change to `lib/` never changes how an old solution builds.

## 11.4 Version manager

`cp self list`, `cp self install 0.4.1`, `cp self use 0.4.1`, and a
`version` pin in `cp.toml` per project. Several versions can be installed
side by side (doc 4b).

## 11.5 Releases

- CI runs the full validation of doc 8 (corpus ports, bug corpus, error
  goldens, benchmarks, judge profiles). Any regression blocks the release.
- The changelog lists every behaviour change with its automatic fix and the
  diagnostic code that flags old code.
- Release binaries are built reproducibly and signed; checksums are
  published with each release. Publishing goes through official release
  channels only, with the user's approval.
