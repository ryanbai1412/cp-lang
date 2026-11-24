# atcoder_dp_g

Ported from `corpus/submissions/4131646.cpp`. Dream: `corpus/problems/atcoder_dp_g/dream.cp`.

## The header lacks its limits and original lines
- **Where:** `port.cp` lines 1-5: `Source:`, `Problem:`, `Solution:` and `Bounds:`, but no
  `// limits: 2000 ms, 1024 MB` and no `// original: corpus/submissions/4131646.cpp`, which the
  other atcoder_dp ports carry. There is also a blank line between the header and the code.
- **What felt wrong:** the header is the port's contract (idiomatic-code/R1). Without
  `limits:` the R10 time budget has to be looked up, and without `original:` the reference
  does too.
- **Goal it hurts:** idiomatic-code/R1.
- **Ideal code:** add the two lines, as in `atcoder_dp_h`.
- **General fix:** the header edit (owner: idioms, this port), landing with the next
  `.cp`-edit commit. The R1 lint does not require `limits:` or `original:`, so this slipped
  through. Routed to Consumers as a lint suggestion: require `limits:` when `meta.json` has a
  time limit.
- **Severity:** low.
- **Owner:** idioms (the header); Consumers (the lint).

## No code gaps
`topo(dag)!.rev` with "every edge leads to an already finished vertex" is the editorial in one
loop. `topo` returns `None` on a cycle; the statement guarantees a DAG, so `!`.

## Measurements
- Tokens: port 119, original 381.
- `--max` (n = m = 1e5, best of 3): port 31.8 ms / 10.8 MB, original 31.7 ms / 11.3 MB. Same
  output.
- Emitted C++: 13,510 bytes.
