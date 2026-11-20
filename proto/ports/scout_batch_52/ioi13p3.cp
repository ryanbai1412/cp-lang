// IOI '13 P3 (Wombats): dynamic R x C grid shortest paths with row/col edge updates.
// Group rows into B=10 blocks in a SegTree of C x C min-plus transition matrices.
// Tropical Optional (?Int) eliminates every 1e9 sentinel in both matrix multiplication
// and left-right / right-left prefix-sum row sweeps.
// original: corpus/submissions/4722414.cpp

const B = 10

struct Wombats:
    r: Int
    c: Int
    h: List<List<Int>>
    v: List<List<Int>>
    nb: Int
    tree: List<List<List<?Int>>>

    fn mul(self, x: List<List<?Int>>, y: List<List<?Int>>) -> List<List<?Int>>:
        var out = List.filled(self.c, List.filled(self.c, none))
        var opt = List.filled(self.c, List.filled(self.c, 0))
        for i in (0..self.c).rev:
            for j in 0..self.c:
                let lo = if j > 0: opt[i][j - 1] else: 0
                let hi = if i + 1 < self.c: opt[i + 1][j] else: self.c - 1
                var best: ?Int = none
                var best_k = lo
                for k in lo..=hi:
                    let cand = x[i][k] + y[k][j]
                    if cand < best:
                        best = cand
                        best_k = k
                out[i][j] = best
                opt[i][j] = best_k
        out

    fn solve_block(self, blk: Int) -> List<List<?Int>>:
        let lo = blk * B
        let hi = min(self.r, lo + B)
        var out = List.filled(self.c, List.filled(self.c, none))
        for i in 0..self.c:
            out[i][i] = 0
            for row in lo..hi:
                for k in 1..self.c: out[i][k] min= out[i][k - 1] + self.h[row][k - 1]
                for k in (0..self.c - 1).rev: out[i][k] min= out[i][k + 1] + self.h[row][k]
                if row + 1 < self.r:
                    for k in 0..self.c: out[i][k] += self.v[row][k]
        out

    fn rebuild_leaf(mut self, blk: Int):
        var o = self.nb + blk
        self.tree[o] = self.solve_block(blk)
        while o > 1:
            o /= 2
            self.tree[o] = self.mul(self.tree[o * 2], self.tree[o * 2 + 1])

var state: ?Wombats = none

@grader
fn init(r: Int, c: Int, h: List<List<Int>>, v: List<List<Int>>):
    var nb = 1
    while nb * B < r: nb *= 2
    var w = Wombats(r, c, h, v, nb, List.filled(2 * nb, []))
    for b in 0..nb: w.tree[nb + b] = w.solve_block(b)
    for o in (1..nb).rev: w.tree[o] = w.mul(w.tree[o * 2], w.tree[o * 2 + 1])
    state = w

@grader
fn changeH(p: Int, q: Int, cost: Int):
    var w = state!
    w.h[p][q] = cost
    w.rebuild_leaf(p / B)
    state = w

@grader
fn changeV(p: Int, q: Int, cost: Int):
    var w = state!
    w.v[p][q] = cost
    w.rebuild_leaf(p / B)
    state = w

@grader
fn escape(v1: Int, v2: Int) -> Int = state!.tree[1][v1][v2]!
