// COCI 2021/22 #4 Parkovi: a tree of towns with road lengths. Build k parks
// so that the farthest any town is from its nearest park is as small as
// possible. Print that distance and the k towns.
// original: corpus/submissions/5120081.cpp
use std.tree.{Rooted}                                               // WISH F1

let n: 1..=2e5 = read()
let k: 1..=n = read()
domain Town = 1..=n
let roads: Tree<Town, (w: 1..=1e9)> = read()                        // WISH R5: read and generated as a tree
let tree = Rooted(roads, Town.first)                                // WISH B14: keeps the road lengths

// The fewest parks that leave every town within `reach` of one, or none if
// that takes more than k. Going up the tree, a park is put down only when
// the farthest unserved town below could not reach anything higher.
fn parks(reach: Int) -> ?List<Town>:
    var built: List<Town> = []
    var far: [Town]?Int = 0                 // farthest town at or below v that no park serves yet
    var near: [Town]?Int = none             // nearest park at or below v
    for v in tree.preorder.rev:             // children before parents
        if let f = far[v] and let d = near[v] and f + d <= reach: far[v] = none
        if let (p, road) = tree.up(v):
            if let f = far[v] and f + road.w > reach:
                built.push(v)
                (near[v], far[v]) = (0, none)
            if let f = far[v]: far[p] max= f + road.w
            if let d = near[v]: near[p] min= d + road.w
        elif far[v] != none:
            built.push(v)
        if built.len > k: return none
    built

let (reach, built) = (0..=2e14).first_some(parks)!                  // WISH R2
print(reach)
let taken = Set(built)
let spare = [t for t in Town if t not in taken]
print(built ++ spare[..k - built.len])                              // any towns will do for the parks left over
