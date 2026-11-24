// Source: NOI 2006 P4 (Maximum Profit), https://dmoj.ca/problem/noi06p4
// limits: 1000 ms, 256 MB
// Problem: N <= 5000 candidate stations with build costs P_i and M <= 50000 user groups;
//   group i pays C_i if both stations A_i and B_i are built. Print the maximum of the total
//   payments minus the total build cost.
// Solution: a maximum-weight closure as a min cut: source -> group (C_i), group -> its two
//   stations (uncapped, never cut), station -> sink (P_i). A cut either drops a group or
//   pays for its stations, so the answer is sum C_i minus the max flow of the network.
// original: corpus/submissions/5032812.cpp
// Bounds: n: 1..=5000; m: 1..=50000; users.a, users.b: 1..=n
let (n, m): (1..=5000, 1..=50000) = read()
domain Station = 1..=n
domain User = 1..=m
let cost: [Station]Int = read()
let users: [User](a: Station, b: Station, pay: Int) = read()

// the network's nodes: the stations, then the groups, then source and sink
domain Node = 0..n + m + 2
let (source, sink) = (Node(n + m), Node(n + m + 1))
fn station(s: Station) -> Node = Node(s.ord)
fn group(u: User) -> Node = Node(n + u.ord)

var net = FlowGraph<Node>()
for s in Station: net.add(station(s), sink, cost[s])
for u in User:
    net.add(source, group(u), users[u].pay)
    net.add(group(u), station(users[u].a), None)
    net.add(group(u), station(users[u].b), None)
print(users.sum(u => u.pay) - net.max_flow(source, sink))
