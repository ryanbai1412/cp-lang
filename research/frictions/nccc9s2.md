# Friction report: nccc9s2

## Dream

```cp
let s: [P]Char = read()
print(sum(abs(2 * k - p) for (k, p) in P.filter(p => s[p] == 'I').items))
```

## Positions of the I's, then one sum

The v0.2 port kept a running `k` and `ans` in a scan over `Int` indices. The rewrite lists
the `I` positions over `domain P = 0..2n` and sums `|2k - p|` over them. `Int(p)` converts the
position for the arithmetic.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (new `gen.py`). The
n = 1e6 max case runs in 0.04 s. Corpus tests pass.
