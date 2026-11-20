// Runnable today/ port of IOI '22 P5 (Insects) for real Rust `cpl build`.
// Stdin: n, then hidden bug color IDs c[0..n] (in 0..n). Simulates move_inside,
// move_outside, press_button, verifies <= 3*n button/move calls, prints min_cardinality(n).
// original: corpus/submissions/4786085.cpp

fn ri() -> Int:
    let x: Int = read()
    return x

fn main_solve():
    let n: Int = read()
    var col: List<Int> = []
    for _ in 0..n: col.push(ri())

    var cnt = [0..n + 1] _ => 0
    var in_box = [0..n] _ => false
    var max_cnt = 0

    fn move_inside(i: Int):
        if not in_box[i]:
            in_box[i] = true
            cnt[col[i]] += 1
            max_cnt max= cnt[col[i]]

    fn move_outside(i: Int):
        if in_box[i]:
            in_box[i] = false
            cnt[col[i]] -= 1
            max_cnt = 0
            for j in 0..n:
                if in_box[j]: max_cnt max= cnt[col[j]]

    fn press_button() -> Int = max_cnt

    var perm: List<Int> = []
    for i in 0..n: perm.push(i)
    // Deterministic Fisher-Yates shuffle
    var seed = 1412345
    var idx = n - 1
    while idx > 0:
        seed = (seed * 1103515245 + 12345) % 2147483647
        let j = seed % (idx + 1)
        perm.swap(idx, j)
        idx -= 1

    var inside: List<Int> = []
    var outside: List<Int> = []
    for i in perm:
        move_inside(i)
        if press_button() > 1:
            move_outside(i)
            outside.push(i)
        else:
            inside.push(i)

    let k = inside.len
    var lo = 1
    var hi = n / k
    while lo < hi:
        let mid = (lo + hi + 1) / 2
        var cur = inside.len
        var added: List<Int> = []
        var rem_out: List<Int> = []
        for i in outside:
            if cur >= mid * k:
                rem_out.push(i)
                continue
            move_inside(i)
            if press_button() <= mid:
                added.push(i)
                cur += 1
            else:
                move_outside(i)
                rem_out.push(i)
        if cur == mid * k:
            lo = mid
            for i in added: inside.push(i)
            outside = rem_out
        else:
            hi = mid - 1
            for i in added: move_outside(i)
            outside = added

    print(lo)

main_solve()
