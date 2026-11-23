// CCC 2019 S4 Tourism: visit n attractions in order, at most k per day, in
// the fewest days possible (ceil(n / k)); each day scores its best attraction.
// Maximise the total score.
//
// Cut the line into blocks of k. With the fewest days, the d-th day must end
// inside block d, so day d starts at j + 1 in block d - 1 and ends at i in
// block d. Its score is max(tail[j], head[i]) where tail[j] is the best of
// j+1.. to the end of j's block and head[i] the best from the start of i's
// block to i. A maximum of two maxima splits into two suffix maxima over j:
//   best[i] = max( max_j (best[j] + tail[j]),  max_j best[j] + head[i] ).
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
@brute(most)
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
