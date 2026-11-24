# Friction Report: noi21p4

No friction left. The `[Block]Map<Int, List<Int>>` with its `contains` / `= []` / `push`
insertion is one `[Block, Half]List<W>`, because a block value is a 16-bit domain `Half`. The
dictionary is one typed `[W]Word` built from the statement's generator. A query block is
`parse_int(base: 16)` of its four digits instead of a hand-written `hex`. The online flip is an
xor mask, and the answer is one short-circuiting `any` over the candidates, which is also the
next query's flip. The bounds `n`, `m` and `k <= 15` are typed.

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++, on the package generator. On its max case (n = 4e5, m = 1.2e5, mostly k = 15) the port
takes 2.52 s and the C++ 1.28 s, with the same output, inside the 3 s limit. The gap is the
per-candidate `bits_apart` over 16 half-words, where the C++ xors four 64-bit words.

## 1. Comparing a candidate costs 16 half-word popcounts
- **Where:** `bits_apart(word, i)` over the `[W, Quarter]U64` dictionary and `any(...)` over the buckets.
- **What felt wrong:** the max case runs in 2.52 s against the accepted C++'s 1.28 s (2x, measurement
  above), inside the 3 s limit but over the 1.1x line.
- **Ideal code:** `any(popcount(word ^ dict[i]) <= k for ...)` over a 256-bit `Word` value.
- **General fix:** a fixed-width bit word (`Bits<256>`) whose xor and popcount compile to four 64-bit
  ops, so the comparison is one expression and as fast as the C++.
- **Owner:** perf (with stdlib for the type). **Severity:** medium.
