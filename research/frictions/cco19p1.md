# Friction report: cco19p1

## Dream

```cp
type Board = [Row, Col]?Player
var memo = Map<(Board, Player), F64>()
fn wins(board: Board, who: Player) -> F64:
    if let known = memo.get((board, who)): return known
    var chances: List<F64> = []
    for p in board.keys if board[p] == Some(who):
        for q in p.adj4 if board[q] != None:
            var next = board
            next[q] = Some(who)
            next[p] = None
            chances.push(1.0 - wins(next, other(who)))
    let best = chances.sorted(by: x => desc(x))
    let take = min(best.len, slack(who))
    let chance = if take == 0: 0.0 else: best[0..take].sum() / F64(take)
    memo[(board, who)] = chance
    chance
```

## Blocked: a Map keyed by a board does not compile

The v0.2 port memoises on a hand base-3 `key(board, who)`. Keying the `Map` on
`(Board, Player)` passes `cpl check`, but the C++ has no `CpHash` for a `Grid`, so g++
rejects it. Repro sent to Core. The port stays at v0.2 until that lands.
