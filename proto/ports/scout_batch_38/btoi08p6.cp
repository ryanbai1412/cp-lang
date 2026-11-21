// Baltic OI '08 P6 (Gloves): choose x left gloves and y right gloves
// minimizing x + y to guarantee at least one same-color pair. A draw (x, y)
// fails iff it lies inside some subset dominance box (sum_S a, sum_{!S} b).
// original: corpus/submissions/5325518.cpp
let n: 1..=20 = read()
domain C = 0..n
let a: [C]Int = read()
let b: [C]Int = read()

// FRICTION 38.5: Subset<C> has no .first (lowest set element / ctz) or .empty,
// so O(2^n) subset-sum DP cannot peel one bit off `s: Subset<C>` without an
// O(n) scan; we double a list instead.
var boxes: List<(Int, Int)> = [(0, b.sum())]
for c in C:
    let cur_len = boxes.len
    for i in 0..cur_len:
        let (sa, sb) = boxes[i]
        boxes.push((sa + a[c], sb - b[c]))

boxes.sort()
var suff_b: [0..boxes.len]Int = [0..boxes.len] i => boxes[i].1
for i in (0..boxes.len - 1).rev:
    suff_b[i] max= suff_b[i + 1]

let (sa_tot, sb_tot) = (a.sum(), b.sum())
var best: ?(Int, Int) = none
for i in 0..boxes.len - 1:
    let x = boxes[i].0 + 1
    let y = suff_b[i + 1] + 1
    if x <= sa_tot and y <= sb_tot:
        if best == none or x + y < best!.0 + best!.1:
            best = (x, y)

print(best!.0)
print(best!.1)
