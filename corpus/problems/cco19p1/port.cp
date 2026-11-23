// CCO 2019 P1, Human Error: Justin (J) and Donald (D) take turns on a
// small board full of their pieces. A move slides one of your pieces onto
// a neighbouring piece and captures it; whoever cannot move loses. Each
// player picks at random among his best few moves. Print the chance that
// Justin, who moves first, wins.
// original: corpus/submissions/3613457.cpp
//
// SKETCH: does not build. `proto/cpc` has no F64, no enum-valued grid
// and no @memo keyed by a whole board.
let n: 1..=13 = read()
let m: 1..=13 = read()                                    // at most 13 cells in all
domain Row = 1..=n
domain Col = 1..=m
domain Cell = (Row, Col)

enum Player: J | D
fn other(p: Player) -> Player: if p == J: D else: J
type Board = [Row, Col]?Player                            // none once a piece has left

let board: Board = read()                                 // WISH L20: a ?Player grid read from letters
let slack: [Player]Int = read()                           // how many of his best moves each may pick from

// The chance that `who`, about to move, wins from here.
@memo                                                     // WISH L23: @memo keyed by an array value
fn wins(board: Board, who: Player) -> F64:
    let chances = sorted_desc(1.0 - wins(board.with(p: None, q: who), other(who))   // WISH L24: board.with(p: v, q: w), a copy with two cells changed
                              for p in Cell if board[p] == who
                              for q in p.around() if board[q] != None)              // WISH L1: p.around() over Dir4
    if chances.len == 0: return 0.0
    mean(chances.take(slack[who]))                        // WISH G21: take; WISH L25: mean

print("{wins(board, J):.3}")                              // WISH G4: decimals
