// Source: NOI 2006 P3 Millennium Worm (千年虫), https://dmoj.ca/problem/noi06p3
// limits: 1000 ms, 128 MB
// Problem: a fossil A gives n rows (n <= 1e6), row i covering cells L_i..=R_i, coordinates
//   <= 1e6. Find a model worm B that corrodes down to A: B covers A row by row, and each side,
//   read down the rows, is a trunk with rectangular feet (the side steps out onto a foot and
//   back in, alternately, the first and last rows on the trunk). Print the fewest cells in B - A.
// Solution: the sides are independent (mirror the left one: a = max L - L), and each picks
//   heights h_i >= a_i minimizing sum(h_i - a_i), where rows keep their height or strict rises
//   and falls alternate. Only heights within 2 of a value in rows i - 2..=i + 2 are tried (<= 25
//   per row). A DP from the bottom keeps states lo (on the trunk, may rise) and hi (on a foot,
//   may fall) per candidate, with running minima over the lower and higher next heights: O(25 n).
// original: corpus/submissions/3868871.cpp
let n: 1..=1_000_000 = read()
domain Col = 0..n
var a: [Col]Int = 0
var b: [Col]Int = 0
for i in Col:
    let (ai, bi): (0..=1_000_000, 0..=1_000_000) = read()
    a[i] = ai
    b[i] = bi
let mx_a = a.max()!
for i in Col: a[i] = mx_a - a[i]

fn cands_at(arr: [Col]Int, i: Int) -> List<Int>:
    let ai = arr[Col(i)]
    var out: List<Int> = []
    for aj in [arr[Col(j)] for j in max(0, i - 2)..=min(n - 1, i + 2)].sorted():
        let from = if out.is_empty: ai else: max(ai, out.last! + 1)
        for k in max(from, aj - 2)..=aj + 2: out.push(k)
    out

fn solve_env(arr: [Col]Int) -> Int:
    var nxt_h = cands_at(arr, n - 1)
    var dp_lo: List<?Int> = [0 for _ in 0..nxt_h.len]
    var dp_hi: List<?Int> = [None for _ in 0..nxt_h.len]

    for i in (0..n - 1).rev:
        let cur_h = cands_at(arr, i)
        let m_nxt = nxt_h.len
        let a_next = arr[Col(i + 1)]
        var new_lo: List<?Int> = List.filled(cur_h.len, None)
        var new_hi: List<?Int> = List.filled(cur_h.len, None)
        var q = 0
        var below: ?Int = None
        for (c, h) in cur_h.items:
            while q < m_nxt and nxt_h[q] < h:
                below min= dp_lo[q] + (nxt_h[q] - a_next)
                q += 1
            new_hi[c] = below
            if q < m_nxt and nxt_h[q] == h: new_hi[c] min= dp_hi[q] + (h - a_next)
        q = m_nxt - 1
        var above: ?Int = None
        for c in (0..cur_h.len).rev:
            let h = cur_h[c]
            while q >= 0 and nxt_h[q] > h:
                above min= dp_hi[q] + (nxt_h[q] - a_next)
                q -= 1
            new_lo[c] = above
            if q >= 0 and nxt_h[q] == h: new_lo[c] min= dp_lo[q] + (h - a_next)

        nxt_h = cur_h
        dp_lo = new_lo
        dp_hi = new_hi

    var ans: ?Int = None
    let a0 = arr[Col(0)]
    for q in 0..nxt_h.len: ans min= dp_lo[q] + (nxt_h[q] - a0)
    ans!

print(solve_env(a) + solve_env(b))
