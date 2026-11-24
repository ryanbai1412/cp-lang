// Source: CCO '22 P1 (Alternating Heights), https://dmoj.ca/problem/cco22p1
// Problem: a line A_1..A_N of students 1..K (N <= 3000, K <= N, students may repeat) and
//   Q <= 1e6 queries x < y: can distinct heights give h[A_x] > h[A_x+1] < h[A_x+2] > ...
//   up to A_y? Print YES or NO for each.
// Solution: a range works iff the taller-than graph of its adjacent pairs, alternating in
//   direction, is acyclic (`consistent`: a topological order exists, O(N + K)), and a range that
//   works still works shrunk. So the furthest y that works from x (`reach[x]`) never moves left
//   as x grows: two pointers make O(N) checks, O(N (N + K)) in all, then O(1) per query.
// original: corpus/submissions/4801881.cpp
// Bounds: n: 2..=3000; k: 2..=n; q: 1..=1e6; line: 1..=k; x, y: 1..=n
let n: 2..=3000 = read()
let k: 2..=n = read()
let q: 1..=1e6 = read()
domain Spot = 1..=n
domain Student = 1..=k
let line: [Spot]Student = read()

fn consistent(x: Spot, y: Spot) -> Bool:
    var taller: [Student]List<Student> = []
    for i in x..y:
        let (a, b) = (line[i], line[i.next!])
        if (i.ord - x.ord) % 2 == 0: taller[a].push(b)
        else: taller[b].push(a)
    topo(taller) != None

var reach = [Spot] x => x
var far = Spot.first!
for x in Spot:
    far max= x
    while let Some(z) = far.next and consistent(x, z): far = z
    reach[x] = far
for _ in 0..q:
    let (x, y): (Spot, Spot) = read()
    assert x < y
    print(yes(y <= reach[x]))
