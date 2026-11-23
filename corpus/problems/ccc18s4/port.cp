// Source: CCC 2018 S4 (Balanced Trees), https://dmoj.ca/problem/ccc18s4
// Problem: a perfectly balanced tree of weight 1 is one node; of weight w >= 2 it is a root with
//   k identical such subtrees (2 <= k <= w), each of weight floor(w / k). Given n <= 1e9, count
//   the perfectly balanced trees of weight n.
// Solution: trees(w) = sum over k = 2..=w of trees(w / k). w / k is constant on O(sqrt w)
//   blocks of consecutive k, so add block size * trees(q) per block. Memoised; only values
//   n / d are reached, about O(n^(3/4)) total.
// As it has to be written today: the blocks of equal quotients are walked by hand. Builds;
//   stress OK against 4329704.cpp.
@memo
fn trees(w: Int) -> Int:
    if w == 1: return 1
    var total = 0
    var k = 2
    while k <= w:
        let q = w / k
        let last = w / q
        total += (last - k + 1) * trees(q)
        k = last + 1
    total
let n: 1..=1e9 = read()
print(trees(n))
