// Source: DMOPC '21 Contest 9 P4 (Ace of Diamonds), https://dmoj.ca/problem/dmopc21c9p4
// Problem: an N x M grid of coins, each H or T, N * M <= 1e6, N and M both odd. Remove every
//   coin one at a time; only a heads coin may be removed, and removing it flips the coins
//   still in the edge-adjacent cells. Print a removal order (r c per line), or -1.
// Solution: on a line, removing the leftmost heads coin each time (an ordered set of ready
//   coins) clears it whenever that is possible. Since M is odd, clearing a whole row flips
//   every coin of its neighbouring rows once and so their heads parity: a row acts like one
//   coin that is heads iff it holds an odd number of heads. Peel the rows as such coins; at
//   its turn a row has been flipped iff its parity was even, so peel its coins inverted
//   then. If the order misses a coin, print -1: O(NM log M).
let rows: 1..=1000 = read()
let cols: 1..=1000 = read()
domain Row = 1..=rows
domain Col = 1..=cols

fn random_grid() -> [Row, Col]Char:
    [Row, Col] (r, c) => if rand(0, 1) == 0: 'H' else: 'T'

let grid: [Row, Col]Char = read(gen: random_grid)

// Part 1 (1-D game): repeatedly remove the leftmost heads-up coin; removing
// a coin flips its neighbours. Returns the removal order, which covers every
// coin iff the line can be cleared.
fn peel<D>(start: [D]Bool) -> List<D>:
    var up = start
    var gone: [D]Bool = false
    var ready = Set<D>()
    for i in D:
        if start[i]: ready.insert(i)
    var order: List<D> = []
    while let u = ready.min():
        ready.remove(u)
        gone[u] = true
        order.push(u)
        for nb in [u.prev, u.next]:
            if let v = nb:
                if not gone[v]:
                    up[v] = not up[v]
                    if up[v]: ready.insert(v) else: ready.remove(v)
    order

@brute(peel)
fn peel_slow<D>(start: [D]Bool) -> List<D>:
    var up = start
    var gone: [D]Bool = false
    var order: List<D> = []
    var progress = true
    while progress:
        progress = false
        for u in D:
            if not progress and up[u] and not gone[u]:
                gone[u] = true
                order.push(u)
                progress = true
                for nb in [u.prev, u.next]:
                    if let v = nb:
                        if not gone[v]: up[v] = not up[v]
    order

// Part 2: a row acts like one coin that is heads iff it has an odd number of heads.
let row_heads: [Row]Bool = [Row] r => count(grid[r, c] == 'H' for c in Col) % 2 == 1

// Part 3: clear rows in peel order. Removing a row flips its neighbours, so a
// row with an even number of heads is cleared with its coins read inverted.
let moves = [
    (r, c) for r in peel(row_heads) for c in peel([Col] c => (grid[r, c] == 'H') == row_heads[r])
]

// Part 4: output the order only if every coin was removed.
if moves.len == Row.len * Col.len:
    for (r, c) in moves:
        print(r, c)
else:
    print(-1)
