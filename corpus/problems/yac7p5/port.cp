// Source: Yet Another Contest 7 P5 (Egg Dropping), https://dmoj.ca/problem/yac7p5
// limits: 7000 ms, 256 MB
// Problem: interactive. Egg i shatters exactly from floor p[i] up, p a uniformly random
//   permutation of 1..=n, n <= 1e4. "? e f" moves from floor c to f and drops egg e: |c - f| + 1
//   moves, plus 20 if the direction reverses. At most 2e5 queries, then "! p"; full score needs <=
//   188000 moves.
// Solution: place(lo, hi, eggs) knows the eggs are exactly those with floors lo..hi: drop all but
//   one at mid - 1 to split them by breaking; the spare goes to the side left one short. Recurse
//   into the half nearer the current floor first, and split small ranges next to it, to keep the
//   elevator moving one way. Each level asks each egg at most once: about n log2 n questions.
// original: corpus/submissions/5705271.cpp; judge: interactor.py
// Bounds: n: 1..=1e4
let n: 1..=1e4 = read()
domain Egg = 1..=n
var floor_of: [Egg]Int = 0
var standing = 0  // the floor of the latest question

fn breaks(egg: Egg, floor: Int) -> Bool:
    standing = floor
    let broke: 0..=1 = ask("?", egg, floor)
    broke == 1

// `eggs` are exactly the eggs whose floors are lo..hi
fn place(lo: Int, hi: Int, eggs: List<Egg>):
    if hi - lo == 1:
        floor_of[eggs[0]] = lo
        return
    let mid = if hi - lo > 3: (lo + hi) / 2 elif standing <= lo: lo + 1 else: hi - 1
    let upper_first = standing < mid - 1
    let spare = eggs.last!
    var (lower, upper) = eggs[..eggs.len - 1].partition(e => breaks(e, mid - 1))
    if upper.len == hi - mid: lower.push(spare) else: upper.push(spare)
    if upper_first:
        place(mid, hi, upper)
        place(lo, mid, lower)
    else:
        place(lo, mid, lower)
        place(mid, hi, upper)

place(1, n + 1, Egg.all)
print("!", floor_of)
