// noi06p3 Millennium Bug: the mechanical v0.3 port (port.cp); a rethought per-row DP is still owed.
// `?Int` + `min=` (zero `INF` sentinels).
// original: corpus/submissions/3868871.cpp
let n: 1..=100_000 = read()
domain Col = 0..n
var a: [Col]Int = 0
var b: [Col]Int = 0
for i in Col:
    let (ai, bi): (Int, Int) = read()
    a[i] = ai
    b[i] = bi
let mx_a = a.max()!
for i in Col: a[i] = mx_a - a[i]

fn cands_at(arr: [Col]Int, i: Int) -> List<Int>:
    var raw: List<Int> = []
    let ai = arr[Col(i)]
    for j in max(0, i - 2)..=min(n - 1, i + 2):
        let aj = arr[Col(j)]
        for k in max(ai, aj - 2)..=aj + 2: raw.push(k)
    var out: List<Int> = []
    for v in raw.sorted():
        if out.is_empty or out.last! != v: out.push(v)
    out

fn solve_env(arr: [Col]Int) -> Int:
    var nxt_h = cands_at(arr, n - 1)
    var dp_lo: List<?Int> = [0 for _ in 0..nxt_h.len]
    var dp_hi: List<?Int> = [None for _ in 0..nxt_h.len]

    for i in (0..n - 1).rev:
        let cur_h = cands_at(arr, i)
        let m_nxt = nxt_h.len
        let a_next = arr[Col(i + 1)]
        var pref_lo: [0..m_nxt]?Int = None
        var suff_hi: [0..m_nxt]?Int = None
        for q in 0..m_nxt:
            if q > 0: pref_lo[q] = pref_lo[q - 1]
            pref_lo[q] min= dp_lo[q] + (nxt_h[q] - a_next)
        for q in (0..m_nxt).rev:
            if q + 1 < m_nxt: suff_hi[q] = suff_hi[q + 1]
            suff_hi[q] min= dp_hi[q] + (nxt_h[q] - a_next)

        var new_lo: List<?Int> = []
        var new_hi: List<?Int> = []
        var q = 0
        for h in cur_h:
            while q < m_nxt and nxt_h[q] < h: q += 1
            var best_lo: ?Int = None
            var best_hi: ?Int = None
            if q > 0: best_hi min= pref_lo[q - 1]
            if q < m_nxt and nxt_h[q] == h:
                let diff = h - a_next
                best_lo min= dp_lo[q] + diff
                best_hi min= dp_hi[q] + diff
                if q + 1 < m_nxt: best_lo min= suff_hi[q + 1]
            elif q < m_nxt:
                best_lo min= suff_hi[q]
            new_lo.push(best_lo)
            new_hi.push(best_hi)

        nxt_h = cur_h
        dp_lo = new_lo
        dp_hi = new_hi

    var ans: ?Int = None
    let a0 = arr[Col(0)]
    for q in 0..nxt_h.len: ans min= dp_lo[q] + (nxt_h[q] - a0)
    ans!

print(solve_env(a) + solve_env(b))
