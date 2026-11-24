// Source: ETSR 2023 R1 P3, https://dmoj.ca/problem/etsr23r1p3
// Problem: multitest; a string s over the first m <= 26 letters. Is every ordering of the m
//   letters a subsequence of s?
// Solution: latest[S] is the latest cut of s after which every ordering of S still fits. An
//   ordering that starts with x needs an x before latest[S - x], so latest[S] = the min over x in
//   S of the last x before latest[S - x], and none if some x has none. The answer is whether
//   latest[all] exists, O(2^m * m). For m >= 22 the accepted C++ answers NO without the table
//   (2^22 cuts would not fit; |s| is too short to hold that many orderings).
// original: corpus/submissions/5494303.cpp
// Bounds: m: 1..=26
@!multitest
let m: 1..=26 = read()
let s: Str = read()
domain Letter = 0..m
domain Pos = 0..s.len

fn solve() -> Bool:
    // before[c][x]: the last x in s before cut c.
    var before: [Pos.cuts][Letter]?Pos = None
    for p in Pos:
        before[p.r] = before[p.l].clone()
        before[p.r][Letter(s[p.ord] - 'a')] = p
    var latest: [Subset<Letter>]?Pos.cuts = None
    latest[Subset<Letter>.empty] = Pos.cuts.last
    for set in Subset<Letter>:
        if set.is_empty: continue
        // Each x in set must be able to start; the min over them, or none when one cannot.
        var best: ?Pos.cuts = Pos.cuts.last
        for x in set:
            if let at = latest[set.without(x)].and_then(c => before[c][x]):
                best = min(best!, at.l)
            else:
                best = None
                break
        latest[set] = best
    latest[Subset<Letter>.full] != None

print(yes(m < 22 and solve()))
