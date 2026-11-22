// IOI 2014 "Friend": people join one at a time through a host and one of three
// protocols; find the maximum-confidence set with no two friends.
enum Protocol = IAmYourFriend | MyFriendsAreYourFriends | WeAreYourFriends

let n = read<1..=1e5>()
domain P = [n]
let conf = read<[P]1..=1e6>()
let join = read<[P.rest](host: P, how: Protocol)>()

var best = [P] p => (out: 0, in: conf[p])
for v in P.rest.rev:
    let (h, b) = (join[v].host, best[v])
    let a = best[h]
    best[h] = match join[v].how:
        IAmYourFriend => (out: a.out + max(b.out, b.in), in: a.in + b.out)
        MyFriendsAreYourFriends => (out: a.out + b.out, in: max(a.in + max(b.out, b.in), a.out + b.in))
        WeAreYourFriends => (out: a.out + b.out, in: max(a.in + b.out, a.out + b.in))
print(max(best[P.first!].out, best[P.first!].in))
