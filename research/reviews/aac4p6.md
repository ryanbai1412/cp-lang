# aac4p6

Score: 3/5
Dream: 3/5

Cold review of `port.cp` against `dream.cp` (the same code bar the header lines), without the C++.
The two-phase idea (simulate min(N, K) hours, then rotate when a cottage is empty) is stated and
the rotate is one line. The hour sweep, though, is the C++ deque bookkeeping, with raw label
arithmetic.

## Checklist failures
- idiomatic-code/R1, R6: the sweep (lines 24-33) carries hour stamps as raw `Int`s:
  `a[I(n - 1 - j)]`, `h >= m - i.label`, `h == m - i.label - 1`, `holes.push_front(-i.label - 1)`.
  The header describes holes but not these offsets (why `-i - 1`, why `m - i - 1` ends a stream),
  so the reader cannot check them.
- idiomatic-code/R2: the seed loop walks `j` and indexes `I(n - 1 - j)` by hand where
  `for x in a.rev.take(m)` says it.

## Owed rewrites
- Seed from `a.rev.take(m)` and name the stamp offset (a `fn stamp(i: I) -> Int`) so each
  comparison reads as "this hole ends before cottage i".

## Language gaps
- 1.21x over the C++ at `--max`, mostly the 1e6-number print: in the friction (codegen).

## Dream notes
The dream is the port, so it keeps the C++ sweep and its unexplained offsets (caps at 3). It
should state the holes stream as an object (hours a cottage receives no elf) with the offsets
derived in the header.
