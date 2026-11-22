# Language server and editors

Status: draft for 0.3.0. Sources: `plan/60-editor.md`, decisions 29,
149. One compiler serves `cpl` and the LSP.

- **Diagnostics** as you type, cheapest first (syntax, names, types,
  domains, borrows, ranges); one error per root cause; each with its
  code, an index link and fixes as code actions. Cost and overflow
  warnings show the estimate on hover.
- **Hover**: the type, where `none` lives, storage width and peak bytes,
  proven ranges, the cost of a call, and representation choices
  (niche, width from a range, specialization chosen by a law).
- **Storage sizes** (decision 149): hover on a type or variable gives its
  size, field storage order, padding and niche; inlay hints after array
  and structure declarations give element size and peak bytes at the
  input bounds (`var dist: [V]?Int  // 8 B x 2e5 = 1.6 MB`); a code lens
  above the program shows total peak memory against the judge profile;
  padding over 20 % and `wide_array` are quick fixes. The numbers come
  from the same computation as `cpl explain`.
- **Inlay hints**: inferred types, parameter names past two positional
  arguments, opt-in live sample values per top-level `let`.
- **Completion** ranked by type (positions of `V` inside `a[` for
  `a: [V]T`, operation values where an `Op<T>` is expected, unimported
  stdlib names with the `use` edit); signature help with bound
  explanations.
- **Navigation and refactoring**: definition, references, rename,
  extract nested fn, inline, add dimension to a DP table, convert array
  to map.
- **Editors**: a VS Code extension (highlighting, run/test/stress
  buttons, debugger, trace panel, judge in the status bar); any LSP
  client works for the rest.
