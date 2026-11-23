# Friction report: ccc20s4

## A letter can't become an enum in one step

- **Where:** `spec.cp:8`, `let who: [Seat]Who = [Seat] i => match s[Int(i)]: 'A' => A ...`
- **What felt wrong:** the statement's alphabet is an enum, but a `Char` has no `to_str`
  (stdlib, filed 2026-11-22) and `parse` works only on `Str`, so the letters are spelled
  twice: once in `Str of "ABC"`, once in the `match`. `Str([c]).parse<Who>()!` checks, but
  `s[i..i + 1].parse<Who>()!` gives a value equal to no variant in `cpl run` (compiler
  worklog), and the C++ for an enum `parse` doesn't compile (missing `<sstream>`, perf worklog).
- **Goal it hurts:** express, submission.
- **Ideal code:** `let who = read<[s.len]Who>()`-style per-letter read, or
  `[Seat] i => s[i].parse<Who>()!`.
- **General fix:** a payload-free enum whose tags are single letters reads from a `Char`
  (`c.parse<Who>()`, or `Str of Who`); `parse` on slices and in codegen fixed (both filed).
- **Severity:** medium.

## Measurements

Max input (n = 1e6, random): spec.cp C++ (g++ -O2) 0.22 s, 39 MB; `cpl run` 43.9 s, 169 MB;
AC 4324014 0.13 s, 12 MB. Emitted C++ 16.3 KB.
