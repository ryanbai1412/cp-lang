// NOI 2009 Poet: t poems. A poem is n sentences (n <= 100000, each at most
// 30 characters with no spaces), to be set in order on lines, sentences on a
// line joined by single spaces. A line of length x costs |x - l|^p
// (l <= 3000000, p <= 10). Print the least total cost and then the lines, or
// "Too hard to arrange" if the cost is above 1e18; then a row of 20 dashes.
//
// best[i], the least cost of the first i sentences, takes its last line
// from some break j, and because the cost is convex the best j never moves
// left as i grows. Keep the breaks that will win somewhere as a list of
// (from which i, break), find where a new break starts to win by binary
// search. Costs are compared as Floats, since they pass 1e18 long before
// the answer is known; the cost printed is added up again in Ints.
// original: corpus/submissions/5108899.cpp
let dashes = "--------------------"

fn power(x: Int, p: Int) -> Int:
    var out = 1
    for _ in 0..p: out *= x
    out

let t: Int = read()
for _ in 0..t:
    let (n, l, p): (Int, Int, Int) = read()
    domain Sentence = 1..=n
    let words: [Sentence]Str = read()
    // ends[i]: where sentence i would end if all were on one line, plus one for the space.
    var ends = [0]
    for s in Sentence: ends.push(ends.last + words[s].len + 1)

    var best = [0.0]
    var cut = [0]           // cut[i]: where the last line of the best setting of i sentences starts
    fn line(j: Int, i: Int) -> Int = abs(ends[i] - ends[j] - 1 - l)
    fn through(j: Int, i: Int) -> Float = best[j] + Float(line(j, i)) ** Float(p)

    var wins: List<(Int, Int)> = [(1, 0)]      // (from, break): `break` is the best from `from` on
    var now = 0
    for i in 1..=n:
        while now + 1 < wins.len and wins[now + 1].0 <= i: now += 1
        cut.push(wins[now].1)
        best.push(through(cut[i], i))
        while wins.last.0 > i and through(i, wins.last.0) <= through(wins.last.1, wins.last.0): wins.pop()
        // The first place after wins.last.0 where breaking at i is at least as good.
        var (lo, hi) = (max(wins.last.0, i) + 1, n + 1)
        while lo < hi:
            let mid = (lo + hi) / 2
            if through(i, mid) <= through(wins.last.1, mid): hi = mid
            else: lo = mid + 1
        if lo <= n: wins.push((lo, i))

    if best[n] > 1e18: print("Too hard to arrange")
    else:
        var breaks = [n]
        while breaks.last > 0: breaks.push(cut[breaks.last])
        breaks.reverse()
        print(sum(power(line(breaks[k], breaks[k + 1]), p) for k in 0..breaks.len - 1))
        for k in 0..breaks.len - 1:
            var text = ""
            for i in breaks[k] + 1..=breaks[k + 1]:
                if not text.is_empty: text.push(' ')
                text.extend(words[Sentence(i)])
            print(text)
    print(dashes)
