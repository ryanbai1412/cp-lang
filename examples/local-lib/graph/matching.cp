//! Maximum bipartite matching (Hopcroft-Karp) with the minimum vertex cover that proves it.
//!
//! `std.flow.max_weight_matching` answers the weighted, perfect case in O(n^3). This is
//! the unweighted one, in O(E sqrt(V)), for two sides of different sizes.

/// A maximum matching between `L` and `R`, where `adj(u)` lists the partners `u` may take.
///
/// - `size`: the number of matched pairs.
/// - `mate[u]` / `back[v]`: the partner of each vertex, `none` if it is left alone.
/// - `cover_l`, `cover_r`: a minimum vertex cover (Konig's theorem): every edge has a
///   marked end and exactly `size` vertices are marked. The unmarked vertices are a
///   maximum independent set.
///
/// O(E sqrt(V)) time, no recursion.
pub fn max_matching<L, R>(adj: L -> List<R>) -> (
    size: Int,
    mate: [L]?R,
    back: [R]?L,
    cover_l: [L]Bool,
    cover_r: [R]Bool,
):
    let nb = [L] u => adj(u)
    var mate: [L]?R = none
    var back: [R]?L = none
    var size = 0
    while true:
        var level = layers(nb, mate, back)
        // is a free right vertex next to a reached left vertex?
        if not any(level[u] != none and any(back[v] == none for v in nb[u]) for u in L): break
        var next: [L]Int = 0
        for root in L:
            if mate[root] != none: continue
            // walk the layers depth-first; `path` holds the left vertices of the walk
            var path = [root]
            while not path.is_empty:
                let u = path.last!
                if next[u] == nb[u].len:
                    level[u] = none  // dead end: never enter it again
                    path.pop()
                    continue
                let v = nb[u][next[u]]
                next[u] += 1
                if let w = back[v]:
                    if level[w] == level[u]! + 1: path.push(w)
                else:
                    // `v` is free: flip the walk, each vertex handing its partner upwards
                    var free = v
                    for i in (0..path.len).rev:
                        let x = path[i]
                        let old = mate[x]
                        mate[x] = free
                        back[free] = x
                        if old != none: free = old!
                    size += 1
                    path = []
    // Konig: from the free left vertices, alternate unmatched and matched edges.
    let level = layers(nb, mate, back)
    var cover_r: [R]Bool = false
    for u in L:
        if level[u] != none:
            for v in nb[u]: cover_r[v] = true
    (size: size, mate: mate, back: back, cover_l: [L] u => level[u] == none, cover_r: cover_r)

/// Breadth-first layers of the left side: free vertices are layer 0, and a matched
/// vertex is one deeper than a left vertex adjacent to its partner.
fn layers<L, R>(nb: [L]List<R>, mate: [L]?R, back: [R]?L) -> [L]?Int:
    var level: [L]?Int = none
    var queue = [u for u in L if mate[u] == none]
    for u in queue: level[u] = 0
    var head = 0
    while head < queue.len:
        let u = queue[head]
        head += 1
        for v in nb[u]:
            if let w = back[v]:
                if level[w] == none:
                    level[w] = level[u]! + 1
                    queue.push(w)
    level

@test
fn size_matches_brute_force_and_the_cover_proves_it():
    domain L = 0..6
    domain R = 0..5
    for _ in 0..40:
        let p = rand(1, 4)
        let adj = [L] u => [v for v in R if rand(0, 4) < p]
        let m = max_matching::<L, R>(u => adj[u])
        // brute force: the best over left vertices in order, by the set of used right vertices
        var best = List.filled(1 << R.K, -1)
        best[0] = 0
        for u in L:
            var after = [b for b in best]
            for used in 0..(1 << R.K):
                if best[used] < 0: continue
                for v in adj[u]:
                    let bit = 1 << v.to_idx
                    if used & bit == 0: after[used | bit] = max(after[used | bit], best[used] + 1)
            best = after
        assert m.size == max(best)
        // the matching is a matching made of real edges
        assert count(m.mate[u] != none for u in L) == m.size
        for u in L:
            if let v = m.mate[u]: assert adj[u].find(v) != none and m.back[v] == u
        // the cover touches every edge and is no larger than the matching
        for u in L:
            for v in adj[u]: assert m.cover_l[u] or m.cover_r[v]
        assert count(m.cover_l[u] for u in L) + count(m.cover_r[v] for v in R) == m.size

/// Are cells `a` and `b` of a 4 by 4 board, numbered row by row, side by side?
fn touching(a: Int, b: Int) -> Bool = abs(a / 4 - b / 4) + abs(a % 4 - b % 4) == 1

@test
fn dominoes_on_a_board_with_holes():
    // cells of one colour on the left, the other colour on the right; a domino is an edge
    let board = ["..#.", "....", ".#..", "...#"]
    domain Cell = 0..16
    let open = [Cell] c => board[c.to_idx / 4][c.to_idx % 4] == '.'
    let dark = [Cell] c => (c.to_idx / 4 + c.to_idx % 4) % 2 == 0
    let m = max_matching::<Cell, Cell>(
        c => [d for d in Cell if open[c] and dark[c] and open[d] and touching(c.to_idx, d.to_idx)]
    )
    assert m.size == 6  // 13 open cells: 6 dark, 7 light
    assert count(
        m.cover_l[c] for c in Cell if open[c] and dark[c]
    ) + count(m.cover_r[c] for c in Cell) == 6
