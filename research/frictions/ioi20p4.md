# Friction report: ioi20p4

## Dream

```cp
@grader fn count_tastiness(x: 1..=1e18, a: [k]0..=1e18) -> Int
domain Bit = 0..60
let s = ([Bit] i => a.get(i) ?? 0 << i).scan(+)
let below = [Bit.cuts] => ...    // below[i] defined from below[j] for j < i
```

## The grader lives in the solution

`count_tastiness` is called q times by the grader. The port reads `q` and each call's input
itself; a `@grader` entry would remove the outer loop and the reads.

## A table over a domain from a shorter list

`a` has `k <= 60` entries but the carries run over all 60 bits, so the table is
`[Bit] i => if Int(i) < k: a[Int(i)] << Int(i) else: 0`. An `a.get(i) ?? 0` that accepts a
`Bit` (or padding a list to a domain) would drop the bound test and the casts.

## A table filled from its own earlier entries

`below` is a list grown with `push`, read by `below[i]` and `below[j]`, and its answer is
`below.last!`. A recursive table over `0..=60` (`[0..=60] i => ...` reading smaller indices)
would type the indices and drop the `!`.

## Verification

30 + 10 random cases agree with the v0.2 port, 30 with the accepted C++ (`orig.cpp`, the AC
grader function plus a stdin `main`), the corpus tests pass, and the max case (q = 1000,
k = 60) is byte-identical to the C++ and runs in 0.01 s.
