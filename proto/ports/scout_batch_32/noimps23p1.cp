// NOI Malaysia PS '23 P1 (DMOJ: noimps23p1) - riding intervals from a start
// Ideal v2 port. original: corpus/submissions/5556659.cpp
//
// Positions 1..=n and m intervals. Standing inside an interval (not on its far
// end) you may ride it to that end, boarding any interval that starts on a
// position you pass. Print every position other than the start where a ride
// can end, going right or going left.
//
// Each position is swept once: an ordered set of unswept positions plus
// `first_ge` replaces the iterator-erase loop of the C++.
let (n, m, start): (1..=2e5, 1..=2e5, 1..=n) = read()
let ivs: [0..m](l: 1..=n, r: l..=n) = read()

// Ends reachable from `from` riding towards larger positions.
fn ends(from: Int, ivs: List<(l: Int, r: Int)>) -> Set<Int>:
    var starting: [1..=n]List<Int> = []
    for (i, iv) in ivs.items: starting[iv.l].push(i)
    var unswept = Set<Int>(1..=n)
    var boarded: [0..m]Bool = false
    var todo: List<Int> = []
    var out = Set<Int>()
    fn board(i: Int):
        if not boarded[i]:
            boarded[i] = true
            todo.push(i)
    for (i, iv) in ivs.items if iv.l <= from and from < iv.r: board(i)
    while let i = todo.pop():
        let (l, r) = ivs[i]
        out.add(r)
        while let x = unswept.first_ge(max(from, l)) and x <= r:
            unswept.remove(x)
            for j in starting[x]: board(j)
    out.remove(from)
    out

// WISH X6: the leftward pass is the same sweep on mirrored coordinates; the
// mirror `n + 1 - x` is written three times because a range cannot be reflected.
fn flip(x: Int) -> Int = n + 1 - x
let right = ends(start, ivs)
let left = ends(flip(start), [(l: flip(iv.r), r: flip(iv.l)) for iv in ivs])
print(" ".join("{x}" for x in right | Set(flip(x) for x in left)))
