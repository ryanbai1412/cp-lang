// IOI 2014 "Friend": people join one at a time through a host and one of three
// protocols; find the maximum-confidence set with no two friends.
enum Protocol = @tag(0) IAmYourFriend | @tag(1) MyFriendsAreYourFriends | @tag(2) WeAreYourFriends

let n = read<2..=1e5>()
domain P = [n]
let conf = read<[P]1..=1e6>()
let joins = read<[n - 1](host: P, how: Protocol)>()  // person v joins in joins[v.ord - 1]

var best = [P] p => (skip: 0, take: conf[p])
for v in P.drop(1).rev:
    let (j, b) = (joins[v.ord - 1], best[v])
    let a = best[j.host]
    best[j.host] = match j.how:
        IAmYourFriend => (skip: a.skip + max(b.skip, b.take), take: a.take + b.skip)
        MyFriendsAreYourFriends => (
            skip: a.skip + b.skip,
            take: max(a.take + max(b.skip, b.take), a.skip + b.take),
        )
        WeAreYourFriends => (skip: a.skip + b.skip, take: max(a.take + b.skip, a.skip + b.take))
print(max(best[P.first!].skip, best[P.first!].take))
