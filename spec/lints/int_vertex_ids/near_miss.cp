let n: 1..=100 = read()
let k: 1..=n = read()
// Slot 0 is the empty prefix, a position of its own: the table is over cuts.
var ways: [0..=n]Int = 0
ways[0] = 1
for i in 1..=n:
    ways[i] = 2 * ways[i - 1] - i
print(ways[k])
// Indexed by a loop from 0, so slot 0 is used.
var seen: [0..=n]Bool = false
for i in 0..=n:
    seen[i] = i % 2 == 0
print(yes(seen[k]))
