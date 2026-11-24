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

## Current port (cold re-read)

Measurements of `corpus/problems/ccc20s4/port.cp` at `--max` (n = 1e6, median of 15):
port 27.5 ms / 11.4 MB, accepted 118.3 ms / 11.2 MB (0.23x time). Output identical.
Tokens: port 394, accepted C++ 1228. Emitted C++: 13580 bytes.

The letter-to-enum friction above no longer applies to this port: it reads `s` as a `Str`
and compares `s[i].ord - 'A'.ord` to a domain ordinal, so it does not need `parse`.

## Friction 2: the read and the Bounds header are unbounded

- Where: `let s: Str = read()`; no `// Bounds:` line.
- What felt wrong: the statement bounds N <= 1e6 and the alphabet; the port states neither.
- Goal it hurts: R1, R5.
- Ideal code: `let s: Str<1..=1e6> = read()` and `// Bounds: s: 1..=1e6 letters of A, B, C`.
- General fix: port edit.
- Severity: medium.
- Owner: idioms.

## Friction 3: a cyclic range count needs three helpers

- Where: `prefix_of`, `upto`, `among`, and the `// WISH C5` comment.
- What felt wrong: "how many of group g sit in seats from..from+len, wrapping" takes a
  two-group prefix table, a derived third group, and a manual wrap in `upto`.
- Goal it hurts: R2, R4.
- Ideal code: `let before = [Group] g => prefix(s.cyclic, c => c == g.letter)` and
  `before[g].sum(at..at + len)`, as the WISH comment says.
- General fix: a std cyclic prefix sum (a `Prefix` over a sequence that answers wrapped
  ranges). Not probed in this pass; Content to confirm whether `prefix` plus a cyclic view
  already composes.
- Severity: low.
- Owner: Content (std/seq.cp, cyclic prefix).
