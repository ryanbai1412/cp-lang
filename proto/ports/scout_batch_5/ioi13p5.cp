// IOI 2013 Robots: a weak robot carries any toy lighter than its limit, a
// small robot any toy smaller than its limit, one toy a minute each. Print
// the fewest minutes to put every toy away, or -1.
// original: corpus/submissions/4688418.cpp (grader-style: a `putaway` function; F20)
let a: 0..=5e4 = read()
let b: 0..=5e4 = read()
let t: 1..=1e6 = read()
let weak: [a]1..=2e9 = read()
let small: [b]1..=2e9 = read()
let toys: [t](weight: 1..=2e9, size: 1..=2e9) = read()

// In `minutes` minutes: weak robots, from the weakest up, each take the
// biggest toys they can lift; small robots, from the largest limit down, take
// the biggest toys left, which must fit.
fn done_in(minutes: Int) -> Bool:
    var left = MaxHeap<Int>()                                       // WISH K9
    var light = sorted(toys).iter                                   // WISH K9: a cursor that survives the loop
    for limit in sorted(weak):
        left.push_all(toy.size for toy in light.take_while(toy => toy.weight < limit))
        left.pop(minutes)                                           // WISH K9: pop up to k
    left.push_all(toy.size for toy in light)
    for limit in sorted(small, desc):                               // WISH K10
        if any(size >= limit for size in left.pop(minutes)): return false
    left.is_empty
print((1..=t).first_where(done_in) ?? -1)                           // WISH K6
