# Bundling and submission

Status: draft for 0.3.0. Sources: `plan/40-tooling.md` §4-5,
`plan/70-judges.md` §7.3, decision 31.

- `cpl build` emits one deterministic C++17 file for the judge profile:
  the stamp and source hash, the used runtime pieces, the reached
  library instances (tree-shaken), the program, `main`. (Rbundle.1)
- Stripped: tests, alts, generators, checkers, interactors, samples,
  `debug(...)`, `@local` items. (Rbundle.2)
- Libraries are inlined from the content-addressed store by pin, so a
  rebuild is byte-identical; `cpl vendor` copies them next to the file.
  Status: planned for Milestone 2 (pins and codemods). (Rbundle.6)
- Size is checked against the profile; `--size` attributes bytes to
  header, runtime, types, library functions, user code and glue. (Rbundle.3)
- `@cpp("...", includes:)` items and `extern fn` judge functions are
  emitted verbatim or as declarations (Rbundle.5).
