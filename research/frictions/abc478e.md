# abc478e friction

There is no `dream.cp`. The port is the ideal, so the `## Dream` is the port as written.

## Dream
`port.cp` as written: `scc(Graph.directed(...))` gives the components in topological order,
`any(t == 1 and comp[u] == comp[v] for (t, u, v) in conds)` is the contradiction test, and one
pass over the arcs sorted by source component sets `a[comp[v]] max= a[comp[u]] + t`.

## No gaps
I re-read every line against the statement and the checklist. The values' range is a type
(`var a: [C]1..=n`), components are their own domain `C`, and the topological order comes
from `scc`, not by hand. There is no dream/port gap to route.

## Measurements

- Judge limits: 2000 ms, 1024 MB, as the port's `limits:` line says; checked against the AtCoder task page, https://atcoder.jp/contests/abc478/tasks/abc478_e.
- Tokens: port 215, original 600.
- `--max` (best of 10): port 0.061 s / 25 MB, `orig.cpp` (not judge-accepted, stress-checked)
  0.143 s / 24 MB. Same output.
- Emitted C++: 20,957 bytes.
