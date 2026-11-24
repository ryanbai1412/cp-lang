# Friction Report: cco18p1

## A Str is not indexed by a domain
- **Where:** `geese_won[g.ord]` beside `geese[g]`.
- **What felt wrong:** the W/L string and the scores share the index space `Game`, but the string
  only takes `Int`, so one lookup is by domain and the other by `.ord`.
- **Ideal:** read the string as `[Game]Char` (or index `Str` by a domain of its length).

## Dream
```cp
let n: 1..=1000 = read()
domain Game = 0..n
let (geese_won, geese): ([Game]Char, [Game]1..=1e6) = read()
let (hawks_won, hawks): ([Game]Char, [Game]1..=1e6) = read()
fn rivals(g: Game, h: Game) -> Bool =
    geese_won[g] != hawks_won[h] and (if geese_won[g] == 'W': geese[g] > hawks[h] else: geese[g] < hawks[h])
var best: [Game.cuts, Game.cuts]Int = 0
for g in Game:
    for h in Game:
        let paired = if rivals(g, h): best[g.l, h.l] + geese[g] + hawks[h] else: 0
        best[g.r, h.r] = max(best[g.l, h.r], best[g.r, h.l], paired)
print(best[Game.cuts.last!, Game.cuts.last!])
```

- Tokens: port 245, original 487
- Max input (`gen.py --max`): port 0.00 s, 10624 KB, original 0.00 s, 7424 KB
