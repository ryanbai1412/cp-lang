# Friction Report: noi14p4

No friction left. The border depths are stepped by hand over the prefix function,
`for j in 1..=s.len: depth.push(depth[pi[j - 1]] + 1)` (port line 17); an earlier note said
they were `Rooted.from_parents`, which the port does not use. The `half` pointer is a second
KMP match, kept explicit because it falls back on two conditions.
