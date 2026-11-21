# Writable-from-docs audit

Model: `gpt-5.4` · Problems: **55** unseen statements from `corpus/statements/` (no existing port in `proto/ports/`).

- **`cpl check` pass rate:** 52/55 (94.5%)
- **`cpl build` pass rate:** 52/55 (94.5%)
- **Full pass (check + build + all samples):** 49/55 (89.1%)

| Problem | `cpl check` | `cpl build` | Samples | Outcome / Error |
| --- | --- | --- | --- | --- |
| `cf2003b` | PASS | PASS | 1/1 | PASS |
| `cf2004b` | PASS | PASS | 1/1 | PASS |
| `cf2004c` | PASS | PASS | 1/1 | PASS |
| `cf2004d` | PASS | PASS | 1/1 | PASS |
| `cf2004e` | PASS | PASS | 1/1 | PASS |
| `cf2005c` | PASS | PASS | 1/1 | PASS |
| `cf2005d` | PASS | PASS | 1/1 | PASS |
| `cf2007b` | PASS | PASS | 1/1 | PASS |
| `cf2007c` | PASS | PASS | 1/1 | PASS |
| `cf2007d` | PASS | PASS | 1/1 | PASS |
| `cf2007e` | PASS | PASS | 1/1 | PASS |
| `cf2013b` | PASS | PASS | 1/1 | PASS |
| `cf2013d` | PASS | PASS | 1/1 | PASS |
| `cf2013e` | PASS | PASS | 1/1 | PASS |
| `cf2020b` | PASS | PASS | 1/1 | PASS |
| `cf2020d` | PASS | PASS | 0/1 | sample 1 TLE (>15s) |
| `cf2020e` | PASS | PASS | 0/1 | sample 1 WA (expected '500000007\n820000006\n280120536\n1', got '0\n0\n0\n0') |
| `cf2025b` | PASS | PASS | 1/1 | PASS |
| `cf2025c` | PASS | PASS | 1/1 | PASS |
| `cf2025d` | PASS | PASS | 3/3 | PASS |
| `cf2025e` | PASS | PASS | 5/5 | PASS |
| `cf2032c` | PASS | PASS | 1/1 | PASS |
| `cf2035b` | PASS | PASS | 1/1 | PASS |
| `cf2035d` | PASS | PASS | 1/1 | PASS |
| `cf2035e` | PASS | PASS | 1/1 | PASS |
| `cf2036b` | PASS | PASS | 1/1 | PASS |
| `cf2036c` | FAIL | — | 0/1 | error[E0102]: expected a value after `and`, found the end of the line |
| `cf2036d` | PASS | PASS | 1/1 | PASS |
| `cf2039c1` | PASS | PASS | 1/1 | PASS |
| `cf2039d` | PASS | PASS | 1/1 | PASS |
| `cf2040b` | PASS | PASS | 1/1 | PASS |
| `cf2040c` | PASS | PASS | 1/1 | PASS |
| `cf2040d` | PASS | PASS | 0/1 | sample 1 WA (expected '2 10 1 6 5\n8 7 12 1 4 6 3', got '1 9 5 3 6\n1 5 13 4 12 7 9') |
| `cf2042b` | PASS | PASS | 1/1 | PASS |
| `abc380d` | PASS | PASS | 3/3 | PASS |
| `abc380e` | PASS | PASS | 1/1 | PASS |
| `abc381d` | PASS | PASS | 3/3 | PASS |
| `abc381e` | FAIL | — | 0/1 | error[E0301]: `pre2` is indexed by 0..=N, but `R` is a domain R |
| `abc382d` | PASS | PASS | 1/1 | PASS |
| `abc382e` | PASS | PASS | 3/3 | PASS |
| `abc383d` | PASS | PASS | 2/2 | PASS |
| `abc383e` | PASS | PASS | 2/2 | PASS |
| `abc384d` | PASS | PASS | 4/4 | PASS |
| `abc384e` | PASS | PASS | 3/3 | PASS |
| `abc385d` | FAIL | — | 0/2 | error[E0204]: no type or domain named `SortedSet` |
| `abc385e` | PASS | PASS | 3/3 | PASS |
| `abc386d` | PASS | PASS | 4/4 | PASS |
| `abc386e` | PASS | PASS | 2/2 | PASS |
| `abc387d` | PASS | PASS | 3/3 | PASS |
| `abc388d` | PASS | PASS | 3/3 | PASS |
| `abc388e` | PASS | PASS | 3/3 | PASS |
| `arc180a` | PASS | PASS | 4/4 | PASS |
| `arc181a` | PASS | PASS | 1/1 | PASS |
| `arc181b` | PASS | PASS | 2/2 | PASS |
| `arc182a` | PASS | PASS | 3/3 | PASS |
