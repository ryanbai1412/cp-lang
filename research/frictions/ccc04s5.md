# ccc04s5: friction report

Port: `corpus/problems/ccc04s5/port.cp`, dream: `corpus/problems/ccc04s5/dream.cp`.

## Measurements

- Tokens: accepted C++ 569; v0.2-form port 318 (14705 bytes of C++).
- Emitted C++ after the re-port: 15913 bytes.
- `--max` (100 x 100 grids, median of 21, g++ -O2): port 7.1 ms / 3.1 MB, accepted 7.7 ms /
  3.5 MB (0.92x time); the v0.2-form port was 12.2 ms on the same input.
- Output: identical to the accepted C++ and to the v0.2-form port.

## Friction 1: the port was the v0.2 translation, not the dream (fixed)

- Where: the whole of `solve_case`: `let neg_inf = -1000000000`, `var c = cols - 1; while c >= 0`,
  the duplicated upward and downward scans, `R(r)` / `C(c)` conversions on every access,
  and the `// Buildable today-form.` header line.
- What felt wrong: `dream.cp` already says this in the language of today: `here: [R]?Int`
  with `None` for unreachable rows, `for c in Col`, one `for step in [-1, 1]` scan and a
  `value(ch)` helper. The port keeps a `-1e9` sentinel compared with `> neg_inf`, walks
  columns right to left with a manual counter (the dream shows left to right is the same
  DP), writes the scan twice, and indexes domains by `Int` conversions.
- Goal it hurts: R2, R4, R5 (sentinel), R6 (`R(r)`), R13.
- Ideal code: `dream.cp` as it stands.
- General fix: re-port from the dream: replace the body with the dream's loop, keep the
  header (minus `Buildable today-form.`), and check it against tests/v02-snapshot as the
  brute. This is a port job, not a language gap; every spelling the dream uses checks today.
- Severity: high (a v03 port carrying a sentinel and duplicated code).
- Fixed: the port is now the dream's body under the original header; `corpus.py verify`
  has 0 findings.
- Owner: none open.

## Friction 2: 1.65x at --max (fixed by the re-port)

- Where: the per-cell `R(r)`, `C(c)` conversions and checks inside the O(m^2 n) scans.
- What felt wrong: each conversion is a range check the accepted C++ does not do; the
  dream's domain loops (`for from in R`, `q in 0..rows` guard) remove most of them.
- Goal it hurts: R10.
- Ideal code: the dream.
- General fix: the re-port in friction 1; measure again after it.
- Severity: low (14 ms against 1000 ms).
- Fixed: re-measured after the re-port at 0.92x.
- Owner: none open.
