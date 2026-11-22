// Problem: ioi19p5 (IOI 2019 P5 - Vision Program)
// Original: corpus/submissions/5664246.cpp
// Blind Critic #5 — First Draft (strictly from docs/one-page-cheat-sheet.md v2.2)

@grader fn add_and(ns: List<Int>) -> Int
@grader fn add_or(ns: List<Int>) -> Int
@grader fn add_xor(ns: List<Int>) -> Int
@grader fn add_not(n: Int) -> Int

@grader fn construct_network(h: Int, w: Int, kv: Int):
    domain Row = 0..h
    domain Col = 0..w
    type Cell = (Row, Col)

    @memo
    fn g_true() -> Int:
        return add_or([p.to_idx for p in Cell])

    @memo
    fn g_false() -> Int:
        return add_not(g_true())

    fn go_cell(cb: fn(Row, Col) -> Int) -> List<Int>:
        var mp = Map<Int, List<Int>>()
        for (r, c) in Cell:
            (mp[cb(r, c)] ??= []).push((r, c).to_idx)
        return [add_or(v) for (_, v) in mp.take().items]

    fn has_k_diff(arr: List<Int>, d: Int) -> Int:
        if d >= arr.len:
            return g_false()
        return add_or([add_and([arr[i], arr[i + d]]) for i in 0..(arr.len - d)])

    fn has_ge_k_diff(arr: List<Int>, d: Int) -> Int:
        if d >= arr.len:
            return g_false()
        var cq: List<Int> = []
        var suff = arr[0]
        for i in d..arr.len:
            suff = add_or([suff, arr[i - d]])
            cq.push(add_and([arr[i], suff]))
        return add_or(cq)

    let c1 = go_cell((r, c) => r.to_idx + c.to_idx)
    let c2 = go_cell((r, c) => r.to_idx - c.to_idx)

    let has_k = add_or([has_k_diff(c1, kv), has_k_diff(c2, kv)])
    let has_gt_k = add_or([has_ge_k_diff(c1, kv + 1), has_ge_k_diff(c2, kv + 1)])
    add_and([has_k, add_not(has_gt_k)])
