// CCO 2017 P5 Professional Network: person i becomes your friend for free once
// you have `need` friends, or at any time for `price`. Befriend all n people
// as cheaply as possible.
// original: corpus/submissions/4575278.cpp
let n: 1..=2e5 = read()
let people: [n](need: 0..=n, price: 1..=1e9) = read()                               // WISH S4 (H1)
let by_need = group(people, by: p => p.need)                                        // WISH S15

// From the most demanding down. `waiting` holds the people who want k friends
// or more and are not paid for. They cannot help each other reach k, so the
// other n - waiting.len people have to be k at least; while they are not, the
// cheapest of those waiting is paid.
var waiting = MinHeap<Int>()
var paid = 0
for k in (0..=n).rev:
    for p in by_need[k]: waiting.push(p.price)
    while n - waiting.len < k: paid += waiting.pop()!                               // WISH S6
print(paid)
