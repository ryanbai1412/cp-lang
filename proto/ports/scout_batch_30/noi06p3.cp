// NOI 2006 P3: given n vertical intervals [a_i, b_i], independently reshape
// the inverted lower envelope (max(a) - a_i) and upper envelope (b_i) into
// alternating valley-peak profiles with minimum total area added.
// Rethought from global 10^5 x 25 x 2 top-down memoization + lower_bound:
// build deduplicated local height candidates per column (at most 25 values)
// and sweep right-to-left with O(25) rolling prefix/suffix minima.
// original: corpus/submissions/3868871.cpp
let n: 1..=100000 = read()
domain Col = 0..n
var a: [Col]Int = 0
var b: [Col]Int = 0
var mx_a = -1000000000
for i in Col:
    let ai: Int = read()
    let bi: Int = read()
    a[i] = ai
    b[i] = bi
    mx_a max= ai
for i in Col:
    a[i] = mx_a - a[i]

const INF = 4000000000000000000

fn unique_sorted(xs: List<Int>) -> List<Int>:
    let s = sorted(xs)
    var out: List<Int> = []
    for v in s:
        if out.len == 0 or out[out.len - 1] != v:
            out.push(v)
    out

fn cands_at(arr: [Col]Int, i: Int) -> List<Int>:
    var raw: List<Int> = []
    let lo = max(0, i - 2)
    let hi = min(n - 1, i + 2)
    let ai = arr[Col(i)]
    for j in lo..=hi:
        let aj = arr[Col(j)]
        let start = max(ai, aj - 2)
        let stop = aj + 2
        if start <= stop:
            for k in start..=stop:
                raw.push(k)
    unique_sorted(raw)

fn solve_env(arr: [Col]Int) -> Int:
    var nxt_h = cands_at(arr, n - 1)
    var dp_lo: List<Int> = [0 for _ in 0..nxt_h.len]
    var dp_hi: List<Int> = [INF for _ in 0..nxt_h.len]

    for i in (0..n - 1).rev:
        let cur_h = cands_at(arr, i)
        let m_nxt = nxt_h.len
        let a_next = arr[Col(i + 1)]
        var pref_lo: [0..m_nxt]Int = INF
        var suff_hi: [0..m_nxt]Int = INF
        for q in 0..m_nxt:
            let c_lo = nxt_h[q] - a_next + dp_lo[q]
            pref_lo[q] = if q == 0: c_lo else: min(pref_lo[q - 1], c_lo)
        for q in (0..m_nxt).rev:
            let c_hi = nxt_h[q] - a_next + dp_hi[q]
            suff_hi[q] = if q == m_nxt - 1: c_hi else: min(suff_hi[q + 1], c_hi)

        var new_lo: List<Int> = []
        var new_hi: List<Int> = []
        var q = 0
        for p in 0..cur_h.len:
            let h = cur_h[p]
            while q < m_nxt and nxt_h[q] < h:
                q += 1
            var best_lo = INF
            var best_hi = INF
            if q > 0:
                best_hi min= pref_lo[q - 1]
            if q < m_nxt and nxt_h[q] == h:
                let diff = h - a_next
                best_lo min= diff + dp_lo[q]
                best_hi min= diff + dp_hi[q]
                if q + 1 < m_nxt:
                    best_lo min= suff_hi[q + 1]
            elif q < m_nxt:
                best_lo min= suff_hi[q]
            new_lo.push(best_lo)
            new_hi.push(best_hi)

        nxt_h = cur_h
        dp_lo = new_lo
        dp_hi = new_hi

    var ans = INF
    let a0 = arr[Col(0)]
    for q in 0..nxt_h.len:
        ans min= nxt_h[q] - a0 + dp_lo[q]
    ans

print(solve_env(a) + solve_env(b))
