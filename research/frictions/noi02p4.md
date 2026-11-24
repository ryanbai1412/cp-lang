# Friction report: noi02p4

## Dream

```cp
fn peaceful(m: Int) -> Bool = not men.pairs().any((a, b) => meet(a, b, m))
```

## Pairs instead of index loops

The v0.2 port walked `for i in 0..n for j in 0..i` over an unindexed `[n]Caveman`. The rewrite
reads the men into a `Man` domain, takes `men.pairs()`, and writes the meeting test as
`linear_congruence(..).is_some_and(..)` instead of an if/else around an optional.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++. The max case runs in
0.13 s. Corpus tests pass.
