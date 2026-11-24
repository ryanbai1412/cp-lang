# Friction Report: cco19p1

The v0.2 port packed the board and the side to move into a base-3 `Int` by hand (`key`) and
memoized on that. Now the memo is keyed by the board itself, one map per side to move
(`[Player]Map<Board, F64>`), the cells are walked with `board.items`, both slacks are read as one
`[Player]1..=16` table, and the best moves are `sorted(by: desc).take(slack[who])`. Keying a Map
by a table needed CpHash for cpl::Grid (and enums) in the C++; that landed with this port.

## A tuple key copies the table on every lookup
- **Where:** `Map<(Board, Player), F64>` with `memo.get((board, who))` is the natural key, but
  the C++ builds a `std::tuple<Grid, Player>` (a heap copy of the board) for each lookup and
  again for the insert: 1.4x the accepted C++'s time at `--max`. One map per side lends the
  board to `get` instead and runs at 1.0x.
- **Ideal:** a lookup by a tuple of places hashes and compares the parts in place (a tuple of
  references), and an insert at a key's last use moves it.

## Dream
```cp
let n: 1..=13 = read()
let m: 1..=13 = read()
domain Row = 0..n
domain Col = 0..m
enum Player = J | D
type Board = [Row, Col]?Player

let letters: [Row, Col]Char = read()
let start: Board = [Row, Col] (r, c) => Some(if letters[r, c] == 'J': J else: D)
let slack: [Player]1..=16 = read()

@memo
fn wins(board: Board, who: Player) -> F64:
    let chances = [1.0 - wins(board.moved(p, q), who.other)
                   for (p, piece) in board.items if piece == Some(who)
                   for q in p.adj4 if board[q] != None]
    chances.sorted(by: desc).take(slack[who]).mean() ?? 0.0

print("{wins(start, J):.3}")
```
