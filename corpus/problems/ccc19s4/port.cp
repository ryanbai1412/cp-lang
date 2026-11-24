// Source: CCC 2019 Senior S4 (Tourism), https://dmoj.ca/problem/ccc19s4
// limits: 1000 ms, 512 MB
// Problem: N attractions visited in order, at most K per day (1 <= K <= N <= 1e6), in the
//   fewest days, ceil(N / K). A day scores the largest a_i (<= 1e9) it visits; print the
//   largest possible total score.
// Solution: cut the line into blocks of K. With the fewest days, day d ends at a cut i
//   inside block d and starts after a cut j of block d - 1 with j >= i - K. Its score is
//   max(tail[j], head[i]), tail[j] the best from j + 1 to the end of j's block and head[i]
//   the best from i's block start to i, so best[i] = max(max_j best[j] + tail[j],
//   max_j best[j] + head[i]): two suffix maxima over the previous block's cuts, O(N).
// original: corpus/submissions/6032792.cpp
let n: 1..=1e6 = read()
let k: 1..=1e6 = read()
assert k <= n
domain Spot = 1..=n
let score: [Spot]1..=1e9 = read()

domain Cut = 0..=n  // number of attractions already seen

fn most(score: [Spot]1..=1e9) -> Int:
    var best: [Cut]Int = 0
    // Suffix maxima over the previous block's cuts j: best[j] and best[j] + tail[j].
    var plain: [Cut]Int = 0
    var tailed: [Cut]Int = 0
    var lo = 0  // current block covers cuts lo+1..=hi
    while lo < n:
        let hi = min(lo + k, n)
        var head = 0
        for i in lo + 1..=hi:
            head max= score[Spot(i)]
            let j = max(i - k, 0)  // earliest cut the day may start from
            best[Cut(i)] = if lo == 0: head else: max(tailed[Cut(j)], plain[Cut(j)] + head)
        var tail = 0
        var run_plain = 0
        var run_tailed = 0
        for j in (lo + 1..=hi).rev:
            run_plain max= best[Cut(j)]
            run_tailed max= best[Cut(j)] + tail
            plain[Cut(j)] = run_plain
            tailed[Cut(j)] = run_tailed
            tail max= score[Spot(j)]
        lo = hi
    best[Cut.last!]

// Direct DP on (days, total): fewest days first, then the largest total.
@alt(most)
fn most_quadratic(score: [Spot]1..=1e9) -> Int:
    var days: [Cut]Int = 0
    var total: [Cut]Int = 0
    for i in 1..=n:
        var d = n + 1
        var t = 0
        var top = 0
        for j in (max(i - k, 0)..i).rev:
            top max= score[Spot(j + 1)]
            let cand_d = days[Cut(j)] + 1
            let cand_t = total[Cut(j)] + top
            if cand_d < d or (cand_d == d and cand_t > t):
                d = cand_d
                t = cand_t
        days[Cut(i)] = d
        total[Cut(i)] = t
    total[Cut.last!]

print(most(score))
