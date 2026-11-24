# Friction report: ioi99p2

## Dream

```cp
let words: [Word]Str = read()
let text: [domain T]Char = read()
let slots = [(w:, j:) for w in Word for j in words[w].keys.rev].group(by: s => words[s.w][s.j])
var start: [Word][..]?T = ...
let best = T.cuts.dp(c => max(best[c.prev], max(len(w) + best[from.l] for each word w ending at c.prev)))
print(best.last, best.argpath)
```

## Ragged per-word tables

`start` is a `[Word]List<?T>` built from each word's length; a ragged table keyed by
`(w: Word, j: words[w].keys)` would type the inner index too.

## Backtracking a cut DP

`best` and `took` run over `T.cuts`, and the walk back uses `T.cuts.at(Int(at) - 1)!` to step
one cut left. A `cut.prev` would say that directly, and a DP helper that records its arg
choice (`argpath`) would drop `took` and the loop.

## Verification

30 random cases pass the checker against the v0.2 port and 30 against the accepted C++, the
max case passes the checker, and the corpus tests pass with the checker.
