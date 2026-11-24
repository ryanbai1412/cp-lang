// Source: NOIP 2020 P1 (Drainage System), https://dmoj.ca/problem/noip20p1
// limits: 1000 ms, 512 MB
// Problem: a DAG of n <= 1e5 junctions; junctions 1..m (m <= 10) each receive one ton of water,
//   and a junction splits all it holds evenly over its 1 to 5 outgoing pipes. Any path has at
//   most 10 inner junctions. For each junction with no outgoing pipe, in order, print the water
//   it ends with as a reduced fraction "p q". Input: n m, then per junction d and d targets.
// Solution: push the water in topological order. A path makes at most 11 splits, each by 1 to
//   5 pipes, so every amount is a whole multiple of 1 / 60^11 (60^11 is about 3.6e19 and has 2^22,
//   3^11 and 5^11): hold each amount as an I128 count of those units and every split divides
//   exactly. Reduce by 2, 3 and 5 at output. O(n + pipes).
// original: corpus/submissions/5445672.cpp

const SPLITS = 11
let unit = I128(60) ** SPLITS

let (n, m) = read<(1..=1e5, 1..=10)>()
domain Junction = 1..=n
let pipes = read<[Junction]List<Junction>>()

var holds = [Junction] u => if u.label <= m: unit else: I128(0)
for u in topo(pipes)!:
    for v in pipes[u]: holds[v] += holds[u] / I128(pipes[u].len)

for u in Junction:
    if pipes[u].is_empty:
        var (num, den) = (holds[u], unit)
        for p in [I128(2), I128(3), I128(5)]:
            while num % p == 0 and den % p == 0: (num, den) = (num / p, den / p)
        print(num, den)
