// Source: CCO 2019 P1 (Human Error), https://dmoj.ca/problem/cco19p1
// Problem: an R x C board (R * C <= 13) with a piece on every cell, each J (Justin) or D
//   (Donald). Justin moves first; a move takes one of your pieces onto an orthogonally
//   adjacent piece (either colour) and captures it; a player who cannot move loses. Each
//   player picks uniformly among his J (resp. D) <= 13 best moves (all, if fewer). Print the
//   chance Justin wins, to 3 decimals.
// Solution: at most 13 cells, so search every reachable board, memoized on the board in base
//   3 plus the side to move. A position's chance for the mover is the mean of 1 - (the
//   opponent's chance) over his best `slack` moves, or 0 with no move.
// original: corpus/submissions/3613457.cpp
let n: 1..=13 = read()
let m: 1..=13 = read()                                    // at most 13 cells in all
domain Row = 0..n
domain Col = 0..m
type Cell = (Row, Col)

enum Player = J | D
fn other(p: Player) -> Player = if p == J: D else: J
type Board = [Row, Col]?Player                            // None once a piece has left

let letters: [Row, Col]Char = read()
let start: Board = [Row, Col] (r, c) => Some(if letters[r, c] == 'J': J else: D)
let (j_slack, d_slack): (Int, Int) = read()              // how many of his best moves each picks from
fn slack(p: Player) -> Int = if p == J: j_slack else: d_slack

fn key(board: Board, who: Player) -> Int:
    var k = if who == J: 0 else: 1
    for r in Row:
        for c in Col:
            k = k * 3 + (if let p = board[r, c]: (if p == J: 1 else: 2) else: 0)
    k

var memo: Map<Int, F64> = Map()

// The chance that `who`, about to move, wins from here.
fn wins(board: Board, who: Player) -> F64:
    let k = key(board, who)
    if let known = memo.get(k): return known
    var chances: List<F64> = []
    for r in Row:
        for c in Col:
            if board[r, c] != Some(who): continue
            let p: Cell = (r, c)
            for q in p.adj4:
                if board[q] == None: continue
                var next = board
                next[q] = Some(who)
                next[p] = None
                chances.push(1.0 - wins(next, other(who)))
    let best = chances.sorted(desc: true)
    let take = min(best.len, slack(who))
    let chance = if take == 0: 0.0 else: best[0..take].sum() / F64(take)
    memo[k] = chance
    chance

print("{wins(start, J):.3}")
