// CCC 2002 S4 Bridge Crossing: a queue of n people crosses a bridge in groups of at most m,
// in queue order; a group takes as long as its slowest member. Minimise the total time
// and print the groups.
// DRAFT 1: greedy, always send a full group of m.
let m: 1..=20 = read()
let n: 1..=100 = read()
domain P = 1..=n

let names = ["ann", "bob", "cy", "dee", "eli", "fay", "gus", "hal"]
fn crowd() -> [P](name: Str, time: 1..=100):
    [P] p => (names[p.idx % names.len], rand(1, 9))

let queue: [P](name: Str, time: 1..=100) = read(gen: crowd)

var total = 0
var groups: List<List<Str>> = []
var i = 0
while i < n:
    let group = [queue[P(j + 1)] for j in i..min(n, i + m)]
    total += max(p.time for p in group)!
    groups.push([p.name for p in group])
    i += m
print("Total Time:", total)
for g in groups: print(g)
