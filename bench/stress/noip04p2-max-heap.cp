// NOIP 2004 Merging Fruit (proto/ports/noip04p2.cp), with a planted wrong
// greedy: the heap pops the largest piles instead of the smallest.
// class: logic
let n: 1..=10000 = read()
let piles: [0..n]1..=20000 = read()

fn effort(piles: List<Int>) -> Int:
    var heap = MaxHeap<Int>()
    for p in piles: heap.push(p)
    var total = 0
    while heap.len > 1:
        let merged = heap.pop()! + heap.pop()!
        total += merged
        heap.push(merged)
    total

@brute(effort)
fn effort_slow(piles: List<Int>) -> Int:
    var left = piles
    var total = 0
    while left.len > 1:
        left = sorted(left)
        let merged = left[0] + left[1]
        total += merged
        left = [merged] + left[2..]
    total

print(effort(piles))
