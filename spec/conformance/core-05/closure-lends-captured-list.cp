// expect: ok
// rule: 05.3.5 13.3.2
// tool: build -o -
// has: "[&xs_2 = xs"
// A lambda reading a `List` parameter lends it, never copies it per call (round 26, B3: O(n^2)).
fn f(xs: List<Int>) -> Int:
    var s = SortedSet<Int>.new()
    var hits = 0
    for i in 0..xs.len:
        if s.first_true(x => x >= xs[i]) != None: hits += 1
    hits
print(f([1, 2, 3]))
