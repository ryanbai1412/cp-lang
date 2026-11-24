# Friction report: cco19p4

## Dream

```cp
var runs: [Value]LiChao<Run, F64> = [Value] v => LiChao(max, over: 0..=count[v])
for v in a.rev:
    ahead[v] -= 1
    runs[v].add(Run(ahead[v] + 1, best), on: 0..=ahead[v] + 1)
    best = runs[v].at(ahead[v])!
```

## Hand Li Chao arena becomes a std `LiChao` per value

The v0.2 port packed one Li Chao tree per value into an offset-indexed arena of three
parallel lists (`has_c`, `cm`, `cb`), with `-1e300` standing for "curve not defined here",
and special-cased k = 2. The rewrite keeps a `[Value]LiChao<Run, F64>`, a `Run` curve with
`at`, and limits each curve to the ranks where it is defined with `add(on:)`, so no sentinel
remains. k = 2 goes through the same DP.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (checker, relative
error). The max case runs in 0.86 s against a 2.75 s limit. Corpus tests pass.
