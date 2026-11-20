// Cheerio Contest 1 S5, clearing blocks around chosen H's
// original: corpus/submissions/5339756.cpp
// Picking an H inside a block scores the block's length and splits it in
// two. Blocks are disjoint and may use at most k H's in total (all the H's
// they contain count, picked or not).
// WISH: Int positions into a string of unstated length (see backlog C2).
let s: Str = read()
let n = s.len
let k: 0..=n = read()
let heads = prefix([0..n] i => if s[i] == 'H': 1 else: 0)

// best score from the block l..r, every pick splitting what is left
@memo
fn clear(l: 0..=n, r: 0..=n) -> Int:
    max(r - l + clear(l, i) + clear(i + 1, r) for i in l..r if s[i] == 'H') ?? 0

// best score from s[i..] when `used` H's are already inside chosen blocks
@memo
fn best(i: 0..=n, used: 0..=k) -> Int:
    if i == n: return 0
    var res = best(i + 1, used)
    for j in i + 1..=n:
        let now = used + heads.sum(i..j)
        if now <= k: res max= clear(i, j) + best(j, now)
    res

print(best(0, 0))
