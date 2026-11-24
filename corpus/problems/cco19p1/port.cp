// Source: CCO 2019 P1 (Human Error), https://dmoj.ca/problem/cco19p1
// limits: 600 ms, 1024 MB
// Problem: an R x C board (R * C <= 13) with a piece on every cell, each J (Justin) or D
//   (Donald). Justin moves first; a move takes one of your pieces onto an orthogonally
//   adjacent piece (either colour) and captures it; a player who cannot move loses. Each
//   player picks uniformly among his J (resp. D) <= 13 best moves (all, if fewer). Print the
//   chance Justin wins, to 3 decimals.
// Solution: at most 13 cells, so search every reachable board, memoized per side to move on the
//   board itself. A position's chance for the mover is the mean of 1 - (the
//   opponent's chance) over his best `slack` moves, or 0 with no move.
// original: corpus/submissions/3613457.cpp
// Bounds: n, m: 1..=13; slack: 1..=16
let n: 1..=13 = read()
let m: 1..=13 = read()  // at most 13 cells in all
domain Row = 0..n
domain Col = 0..m
type Cell = (Row, Col)

enum Player = J | D
fn other(p: Player) -> Player = if p == J: D else: J
type Board = [Row, Col]?Player  // None once a piece has left

let letters: [Row, Col]Char = read()
let start: Board = [Row, Col] (r, c) => Some(if letters[r, c] == 'J': J else: D)
let slack: [Player]1..=16 = read()  // how many of his best moves each picks from

var memo: [Player]Map<Board, F64> = [Player] _ => Map()

// The chance that `who`, about to move, wins from here.
fn wins(board: Board, who: Player) -> F64:
    if let known = memo[who].get(board): return known
    var chances: List<F64> = []
    for (p, piece) in board.items:
        if piece != Some(who): continue
        for q in p.adj4:
            if board[q] == None: continue
            var next = board.clone()
            next[q] = Some(who)
            next[p] = None
            chances.push(1.0 - wins(next, other(who)))
    let best = chances.sorted(by: x => desc(x)).take(slack[who])
    let chance = if best.is_empty: 0.0 else: best.sum() / F64(best.len)
    memo[who][board] = chance
    chance

print("{wins(start, J):.3}")
