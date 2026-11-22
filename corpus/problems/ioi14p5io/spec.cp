// IOI 2014 "Friend": people join one at a time through a host and one of three
// protocols; find the maximum-confidence set with no two friends.
enum Protocol = IAmYourFriend | MyFriendsAreYourFriends | WeAreYourFriends

let n = read<1..=1e5>()
domain P = [n]
let conf = read<[P]1..=1e6>()
let join = read<[P](host: P, how: Protocol)>()

var best = [P] p => (skip: 0, take: conf[p])
for v in P.drop(1).rev:
    let (h, b) = (join[v].host, best[v])
    let a = best[h]
    best[h] = match join[v].how:
        IAmYourFriend => (skip: a.skip + max(b.skip, b.take), take: a.take + b.skip)
        MyFriendsAreYourFriends => (skip: a.skip + b.skip, take: max(a.take + max(b.skip, b.take), a.skip + b.take))
        WeAreYourFriends => (skip: a.skip + b.skip, take: max(a.take + b.skip, a.skip + b.take))
print(max(best[P.first!].skip, best[P.first!].take))
