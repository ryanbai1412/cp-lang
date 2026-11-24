// Source: NOI 2006 P4 (Maximum Profit), https://dmoj.ca/problem/noi06p4
// Problem: N <= 5000 candidate stations with build costs P_i and M <= 50000 user groups;
//   group i pays C_i if both stations A_i and B_i are built. Print the maximum of the total
//   payments minus the total build cost.
// Solution: a maximum-weight closure as a min cut: source -> group (C_i), group -> its two
//   stations (capacity 1e9, never cut), station -> sink (P_i). A cut either drops a group or
//   pays for its stations, so the answer is sum C_i minus the max flow of the network.
// As it runs today on the reference interpreter (`cp.py lib-run`): the network's vertices
// are one numbered domain with offsets kept by hand (K11).
// original: corpus/submissions/5032812.cpp
// Bounds: n: 1..=5000; m: 1..=50000; users.a, users.b: 1..=n

let n: 1..=5000 = read()
let m: 1..=50000 = read()
domain Station = 1..=n
domain User = 1..=m
let cost: [Station]Int = read()
let users: [User](a: Station, b: Station, pay: Int) = read()

domain Node = 0..n + m + 2
let source = Node(n + m)
let sink = Node(n + m + 1)
fn station(s: Station) -> Node: Node(s.label - 1)
fn user(u: User) -> Node: Node(n + u.label - 1)

var net = FlowGraph<Node>()
for s in Station: net.add(station(s), sink, cost[s])
for u in User:
    net.add(source, user(u), users[u].pay)
    net.add(user(u), station(users[u].a), 1e9)
    net.add(user(u), station(users[u].b), 1e9)
print(sum(users[u].pay for u in User) - net.max_flow(source, sink))
