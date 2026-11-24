// expect: ok, output "ok"
// input: "1 300\n"
// model: shuffle
use std.rand.{Rng}

// Free `shuffle` against a brute force: each run leaves a permutation of its input (same multiset,
// same length), and over enough runs of three distinct values every one of the 6 orders turns up.
let seed = read<Int>()
let n = read<Int>()
var rng = Rng(seed)
var bad = ""
var seen = Set<List<Int>>()
for k in 0..n:
    if bad != "": break
    let xs = [rng.int(-9, 9) for _ in 0..rng.int(0, 7)]
    var got = xs.clone()
    shuffle(mut got)
    if got.len != xs.len or got.sorted() != xs.sorted(): bad = "op {k}: shuffle({xs}) = {got}"
    var three = [1, 2, 3]
    shuffle(mut three)
    seen.insert(three)
if bad == "" and n >= 200 and seen.len != 6: bad = "only {seen.len} of 6 orders of [1, 2, 3] in {n} runs"
print(if bad == "": "ok" else: bad)
