# Friction report: ioi12p2

There is no dream.cp; the ideal is below. The port follows the editorial. Up to the first ring of degree 3, a std `Dsu<Ring>` counts cycles. After it, the hub and its three neighbours (`Cand = 0..4`) each keep a `Dsu` and degrees of the graph without them, fed every link so far and each new one.

## Dream

```cp
@grader mod rings:
    fn init(n: Int)
    fn link(a: Ring, b: Ring)
    fn count_critical() -> Int
struct Without:
    var dsu: Dsu<Ring>
    var deg: [Ring]Int
    var chains: Bool
var without: [Cand]Without
```

## Each candidate's state is three parallel arrays
- **Where:** lines 468-470: `without: [Cand]Dsu<Ring>`, `deg_without: [Cand, Ring]Int`, `chains: [Cand]Bool`, all updated together in `link_without`.
- **What felt wrong:** "the graph without candidate k" is one object, kept as three arrays indexed by the same key.
- **Goal it hurts:** express, coherence.
- **Ideal code:** the dream's `struct Without` and `var without: [Cand]Without`.
- **General fix:** none in the language; a struct of a `Dsu` and a `[Ring]Int` compiles today. It is a rewrite owed by the port.
- **Severity:** low.

## The grader interface becomes a stdin encoding
- **Where:** lines 449-451 and 504-507: calls arrive as `u v` lines with `-1 -1` meaning CountCritical, read as `(-1..=n - 1, -1..=n - 1)`.
- **What felt wrong:** the -1 is a sentinel the port invented to replay the grader's calls. The submitted program has to be the three functions `Init`/`Link`/`CountCritical`.
- **Goal it hurts:** submission, hard-to-get-wrong.
- **Ideal code:** the dream's `@grader mod` with `init`, `link` and `count_critical`.
- **General fix:** grader modules with setup-sized state, the same gap as research/frictions/ioi25_obstacles.md. Owner: core/spec.
- **Severity:** medium.

## Measurements

- **Tokens:** port 513, accepted C++ 1143 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.16 s / 70 MB, accepted C++ 0.42 s / 71 MB, with the same output.
- **Emitted C++:** 21480 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
