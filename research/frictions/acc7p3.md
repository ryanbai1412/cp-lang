# acc7p3 friction

There is no `dream.cp`. The port is the ideal, so the `## Dream` is the port as written.

## Dream
`port.cp` as written: the existing friendships as zero-cost links, plus the neighbouring pairs
by value (`U.sort(by: u => f[u]).adjacent`), fed to `kruskal(links).total`.

## No gaps
The exchange argument that keeps only neighbouring pairs is in `Solution:`. The MST comes from
std, and users are the domain `U`. The statement says the values increase with i, so
`U.sort(by: u => f[u])` is U in order. Sorting anyway keeps the port correct if that
guarantee is ever read loosely, at O(n log n), so it stays. There is no dream/port gap.

## Measurements

- Judge limits: 750 ms, 256 MB, as the port's `limits:` line says; checked against the DMOJ API (`/api/v2/problem/acc7p3`), https://dmoj.ca/problem/acc7p3.
- Tokens: port 148, original 779.
- `--max` (best of 10): port 0.031 s / 14 MB, accepted C++ 0.065 s / 17 MB. Same output.
- Emitted C++: 21,498 bytes.
