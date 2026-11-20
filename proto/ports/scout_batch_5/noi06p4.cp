// NOI 2006 Profit: building station s costs cost[s]; user u pays once both of
// their stations are built. Print the best total profit.
// original: corpus/submissions/5032812.cpp
use std.flow.{max_closure}                                          // WISH F1, K12

let n: 1..=5000 = read()
let m: 1..=50000 = read()
domain Station = 1..=n
domain User = 1..=m
let cost: [Station]1..=100 = read()
let users: [User](a: Station, b: Station, pay: 1..=100) = read()

// Choosing a user forces both stations: the best set that is closed under "needs".
enum Item:                                                          // WISH K11: an enum of domains as an index
    Serve(u: User)
    Build(s: Station)
fn gain(i: Item) -> Int:
    match i:
        Serve(u) => users[u].pay
        Build(s) => -cost[s]
fn needs(i: Item) -> yields Item:
    if let Serve(u) = i:
        yield Build(users[u].a)
        yield Build(users[u].b)
print(max_closure(gain, needs).value)
