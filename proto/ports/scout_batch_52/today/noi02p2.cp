// Runnable today/ port of NOI '02 P2 for real Rust `cpl build`.
// Stdin: cnt_n, cnt_m, hidden types t[0..n] (0=liar, 1=truth, 2=wild), and wild_ans[0..n]
// (wild card's fixed answer when asked about each person). Simulates Ask(i, j, 0) and prints recovered types.
// original: corpus/submissions/4793917.cpp

fn ri() -> Int:
    let x: Int = read()
    return x

fn main_solve():
    let cnt_n: Int = read()
    let cnt_m: Int = read()
    let n = cnt_n + cnt_m + 1
    var kind: List<Int> = []
    var wild_ans: List<Int> = []
    for _ in 0..n: kind.push(ri())
    for _ in 0..n: wild_ans.push(ri())

    // 0 = truth, 1 = liar, 2 = wild
    fn ask_ring(i: Int, j: Int) -> Int:
        if kind[i] == 2: return wild_ans[j]
        if kind[i] == 0: return if kind[j] == 0: 1 else: 0
        return if kind[j] == 0: 0 else: 1

    var dir_r = [0..n] _ => 0
    var dir_l = [0..n] _ => 0
    for p in 0..n:
        dir_r[p] = ask_ring(p, (p + 1) % n)
        dir_l[p] = ask_ring(p, (p + n - 1) % n)

    var wild = 0
    for p in 0..n:
        if dir_r[p] != dir_l[(p + 1) % n]:
            wild = p
            break

    var ans = [0..n] _ => 0
    ans[wild] = 2
    var p = wild - 1
    while p >= 0:
        let next_is_zero = if ans[p + 1] == 0: 1 else: 0
        ans[p] = if dir_r[p] != next_is_zero: 1 else: 0
        p -= 1
    for q in wild + 1..n:
        let prev_is_zero = if ans[q - 1] == 0: 1 else: 0
        ans[q] = if dir_l[q] != prev_is_zero: 1 else: 0

    for i in 0..n: print(ans[i])

main_solve()
