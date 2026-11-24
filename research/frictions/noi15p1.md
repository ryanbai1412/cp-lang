# Friction report: noi15p1

The port is now the dream with the statement bounds as read types (`q: 1..=1e5`, `i, j: 1..=1e9`, `e: 0..=1`). The `// Bounds:` header was added, and the inequality filter is written `c.e == 0`. The v0.2 port read `(Int, Int, Int)` triples into parallel `rules`/`raw` lists and had no bounds.

## Dream

`dream.cp`, which the port now matches. There is no dream/port gap.

## gen.py `--max` took over a minute
- **Where:** `gen.py`, the inequality branch picked another class with `r.choice([c for c in by if c != cls[i]] ...)`, a list of up to 5000 classes per row over 1e6 rows. `corpus.py make` and any `--max` timing waited on it.
- **What felt wrong:** a generator slower than the solution it tests hides max cases.
- **Fix (done):** pick from a list of classes built once per instance; the rare pick of the same class gives an equality, which the instance's answer already allows. `gen.py --max` now takes 5 s.
- **Owner:** done in this commit.
- **Severity:** low.

## Measurements

- **Tokens:** port 173, accepted C++ 590 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.64 s / 12 MB, accepted C++ 0.85 s / 35 MB, with the same output.
- **Emitted C++:** 21094 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
