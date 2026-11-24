// IOI 2014 "Friend": people join one at a time through a host and one of three
// protocols; find the maximum-confidence set with no two friends.
//
// Undo the joins newest first, folding each newcomer v into its host h: a
// person's state is the best total with them left out (`free`) or taken.
enum Protocol = @tag(0) IAmYourFriend | @tag(1) MyFriendsAreYourFriends | @tag(2) WeAreYourFriends

let n = read<2..=1e5>()
domain P = [n]
let conf = read<[P]1..=1e6>()
let joins = read<[n - 1](host: P, how: Protocol)>()

var best = [P] p => (free: 0, taken: conf[p])
for (k, j) in joins.items.rev:
    let v = P(k + 1)
    let (a, b) = (best[j.host], best[v])
    let either = max(b.free, b.taken)
    best[j.host] = match j.how:
        IAmYourFriend => (free: a.free + either, taken: a.taken + b.free)
        MyFriendsAreYourFriends => (free: a.free + b.free, taken: max(a.taken + either, a.free + b.taken))
        WeAreYourFriends => (free: a.free + b.free, taken: max(a.taken + b.free, a.free + b.taken))
let root = best[P.first!]
print(max(root.free, root.taken))
