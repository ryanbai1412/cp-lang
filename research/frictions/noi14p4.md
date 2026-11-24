# Friction Report: noi14p4

No friction left: the border depths are `Rooted.from_parents` over the prefix function, as
`L0567` asked (they used to be hand-stepped `depth[i + 1] = depth[pi[i]] + 1`). The `half`
pointer is a second KMP match, kept explicit because it falls back on two conditions.
