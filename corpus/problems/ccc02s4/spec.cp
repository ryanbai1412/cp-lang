// ccc02s4 Bridge Crossing: people cross in order, at most m at a time, and a group takes as
// long as its slowest member. Split the line into groups to minimize the total.
let (m, n) = read<(1..=20, 1..=100)>()
domain P = 0..n
let people = read<[P](name: Str, t: 1..=1000)>()

fn cost(c: P.cuts, e: P.cuts) -> Int = people[c..e].map(p => p.t).max()! + total(e)
@memo(array)
fn total(c: P.cuts) -> Int:
    if c == P.cuts.last!:
        return 0
    (c.next!..).take(m).map(e => cost(c, e)).min()!

print("Total Time:", total(P.cuts.first!))
var c = P.cuts.first!
while c != P.cuts.last!:
    let e = (c.next!..).take(m).min(by: e => cost(c, e))!
    print(people[c..e].map(p => p.name))
    c = e
