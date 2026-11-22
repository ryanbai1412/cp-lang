# Tools and setup (`/tools`, `/setup`)

Status: proposed.

## `/tools`: overview

One page: the contest loop as a picture (write -> check -> run ->
test -> stress -> trace -> build -> submit), each step naming its
command and linking to its page. Then the editor support and judge
profiles.

## `/tools/cli/<command>`: CLI reference

One page per command, generated from `cpl <cmd> --help` (or a new `cpl
help --json`), plus a hand-written "when to use it" paragraph and one
worked example per page. Commands today *(verified, `cpl --help`)*:
`check`, `fix`, `run`, `judge`, `build`, `judges`, `submit`, `fmt`,
`lint`, `explain`, `doc`, `new`, `test`, `stress`, `bench`, `profile`,
`cover`, `watch`, `lsp`, `dap`. The generator fails the build when a
command exists without a page or a page has no command.

## `/tools/workflows/<name>`: workflows

Hand-written, one task each, every command shown with its output:

- **From samples to AC**: `cpl new`, `cpl watch`, `cpl test`, `cpl
  submit`.
- **Stress a greedy**: `@brute`, `cpl stress`, reading a shrunk case.
- **Find the first wrong value**: `cpl run --trace`, the trace viewer,
  the debugger (`cpl dap`).
- **Is it fast enough?**: `cpl bench`, `cpl profile`, `cpl explain`.
- **Interactive problems locally**: `@interactor`, `cpl run`, `cpl
  judge`.
- **Problems with many answers**: `@checker`.
- **Coverage of your tests**: `cpl cover`.

## `/tools/editors`: editor support

What `cpl lsp` gives (diagnostics, hover, completion, signature help,
go to definition, references, rename, symbols, formatting, quick fixes,
inlay hints; *verified*, `editors/README.md`) and one section per
editor: VS Code, Neovim/Vim, Helix, Zed, Emacs, JetBrains, other LSP
clients. Each section is the setup from `editors/<editor>/README.md`,
included at build time so there's one source.

## `/tools/judges`: judge profiles

Generated from `cpl judges` and `spec/tools/judge-profiles.md`: each
profile's compiler, flags, stack and source limits, with why each
matters. Plus **Submitting to a judge**: which judges take the generated
C++17, which compiler to pick on each, the 64 KB limit and the 32 KB
warning, what to do if a judge rejects the file.

## `/setup`: the setup checklist

One page, each step a checkbox kept in `localStorage`, with a "verify"
command and its expected output.

1. **Install `cpl`**. Tabs: macOS/Linux installer script, Windows,
   from source. Until prebuilt binaries exist, only the from-source tab
   is live (`cargo install --path crates/cpl-cli`, *verified*); the
   other tabs say "coming soon". Upgrade and uninstall below.
2. **Install the editor extension**. One tab per editor. Designed for
   one-click installs (Marketplace, Open VSX for Cursor/Windsurf/
   VSCodium, the Zed and JetBrains registries), but until they're
   published each tab shows today's from-source install (Ryan: design
   for it, don't build it yet). A short clip of diagnostics, hover and
   the Run/Test/Stress lenses.
3. **Check it works**. Today: `cpl --version` and opening a file in the
   editor. Later: `cpl doctor` (new) checks the binary, version, `g++`,
   judge profile and the editor's LSP connection.
4. **Pick your judge**: `cpl judges`, setting the default.
5. **Set up the contest loop**: `cpl new`, `watch`, `test`, `submit`;
   Competitive Companion once `cpl` has its listener (new).
6. **Extras**: shell completions, a personal library folder, an AI
   assistant rules snippet pointing at `/llms.txt`.

Nudges to Setup: after the first course AC, in the playground header,
at the end of each quickstart, in the docs footer.
