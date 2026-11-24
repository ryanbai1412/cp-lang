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
// Bounds: n: 1..=1e6; k: 1..=n; score: 1..=1e9
let n: 1..=1e6 = read()
let k: 1..=n = read()
domain Spot = 1..=n
type Score = 1..=1e9
let score: [Spot]Score = read()

domain Cut = 0..=n  // number of attractions already seen

fn most(score: [Spot]Score) -> Int:
    var best: [Cut]Int = 0
    // Suffix maxima over the previous block's cuts j: best[j] and best[j] + tail[j].
    var plain: [Cut]Int = 0
    var tailed: [Cut]Int = 0
    for lo in (0..n).step(k):
        let block = lo + 1..=min(lo + k, n)  // this block's cuts
        var head = 0
        for i in block:
            head max= score[Spot(i)]
            let j = Cut(max(i - k, 0))  // the earliest cut the day may start from
            best[Cut(i)] = if lo == 0: head else: max(tailed[j], plain[j] + head)
        var (tail, run_plain, run_tailed) = (0, 0, 0)
        for j in block.rev:
            run_plain max= best[Cut(j)]
            run_tailed max= best[Cut(j)] + tail
            plain[Cut(j)] = run_plain
            tailed[Cut(j)] = run_tailed
            tail max= score[Spot(j)]
    best[Cut.last!]

// Direct DP over (days, -total) per cut: fewest days first, then the largest total.
@alt(most)
fn most_quadratic(score: [Spot]Score) -> Int:
    var plan: [Cut](days: Int, loss: Int) = (days: 0, loss: 0)
    for i in 1..=n:
        plan[Cut(i)] = min((days: plan[Cut(j)].days + 1,
                            loss: plan[Cut(j)].loss - max(score[Spot(t)] for t in j + 1..=i)!)
                           for j in max(i - k, 0)..i)!
    -plan[Cut.last!].loss

print(most(score))
