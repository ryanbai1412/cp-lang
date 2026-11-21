// User-defined fn with keyword-labelled parameter (`while:`) — no stdlib privilege.
fn take_prefix(xs: List<Int>; while ok: Int -> Bool) -> List<Int>:
    var out: List<Int> = []
    for x in xs:
        if not ok(x): break
        out.push(x)
    out
print(take_prefix([1, 2, 4, 3, 1], while: x => x < 4))

// `drain(after:/before:, while:)` parity across Set, SortedSet, and SortedBag.
var s_drain: Set<Int> = Set()
for x in [1, 3, 5, 7, 9, 12]: s_drain.add(x)
let d1 = s_drain.drain(after: 3, while: x => x <= 9)
let d2 = s_drain.drain(before: 12, while: x => x >= 2)
let d3 = s_drain.drain(while: x => x <= 5)
print(d1, d2, d3, s_drain.sorted())

var ss_drain = SortedSet<Int>.of([1, 3, 5, 7, 9, 12])
let sd1 = ss_drain.drain(after: 3, while: x => x <= 9)
let sd2 = ss_drain.drain(before: 12, while: x => x >= 2)
let sd3 = ss_drain.drain(while: x => x <= 5)
print(sd1, sd2, sd3, [x for x in ss_drain])

var sb_drain = SortedBag<Int>.of([2, 4, 4, 6, 8, 8, 10])
let bd1 = sb_drain.drain(after: 2, while: x => x <= 6)
let bd2 = sb_drain.drain(before: 10, while: x => x >= 8)
print(bd1, bd2, [x for x in sb_drain.range(..)])
