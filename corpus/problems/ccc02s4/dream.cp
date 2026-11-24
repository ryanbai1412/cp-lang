// CCC 2002 S4 Bridge Crossing: people cross in order, groups of at most m at a time, a group takes
// as long as its slowest member. Minimize the total and print the groups.
let (m, n) = read<(1..=20, 1..=100)>()
domain P = 0..n
let people = read<[P](name: Str, t: 1..=1000)>()

let times = [p.t for p in people]
let names = [p.name for p in people]

// best(c): the least time to get everyone from position c onward across, and where c's group ends.
@memo(array)
fn best(c: 0..=100) -> (total: Int, end: Int):
    if c == n:
        return (total: 0, end: n)
    [(total: times[c..e].max()! + best(e).total, end: e) for e in c + 1..=min(c + m, n)].min()!

print("Total Time:", best(0).total)
var c = 0
while c != n:
    let e = best(c).end
    print(names[c..e])
    c = e
