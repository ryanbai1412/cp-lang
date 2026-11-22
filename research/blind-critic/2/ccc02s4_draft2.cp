// CCC 2002 S4 Bridge Crossing: a queue of n people crosses a bridge in groups of at most m,
// in queue order; a group takes as long as its slowest member. Minimise the total time
// and print the groups.
// DRAFT 2: the DP, but ties keep the LAST best cut (`<=`), so the groups differ from the judge's.
// best[i] = least time for the people from position i on; cut[i] = where the first of
// those groups ends (the shortest such group on ties, as the original prints).
let m: 1..=20 = read()
let n: 1..=100 = read()
domain P = 1..=n

let names = ["ann", "bob", "cy", "dee", "eli", "fay", "gus", "hal"]
fn crowd() -> [P](name: Str, time: 1..=100):
    [P] p => (names[p.idx % names.len], rand(1, 9))

let queue: [P](name: Str, time: 1..=100) = read(gen: crowd)

// positions are 0..=n here: position i sits before person i + 1
var best: [0..=n]Int = 0
var cut: [0..=n]Int = n
for i in (0..n).rev:
    var slowest = 0
    var least: ?Int = none
    for j in i + 1..=min(n, i + m):
        slowest max= queue[P(j)].time
        let t = slowest + best[j]
        if least == none or t <= least!:
            least = t
            cut[i] = j
    best[i] = least!
print("Total Time:", best[0])
var i = 0
while i < n:
    print([queue[P(j + 1)].name for j in i..cut[i]])
    i = cut[i]
