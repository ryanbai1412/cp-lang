// Source: CCO 2020 P4, Travelling Salesperson, https://dmoj.ca/problem/cco20p4
// Problem: n <= 2000 buildings, every pair joined by a red or a blue road. For each start,
//   print a route of minimum length (buildings visited, repeats counted) that visits every
//   building and switches road colour at most once. A route of length M <= 2K, K optimal,
//   scores partially: floor(8 + 8 (2K - M) / (K - 1)) of 25.
// Solution: one colour connects every building (a graph or its complement is connected).
//   `tour` builds the closed DFS walk over a spanning tree in that colour, with no colour
//   switch; dropping the return leaves 2n - 2 stops through every building, and each start
//   prints that cycle rotated to begin at it. O(n^2) to build, O(n^2) output. This is the
//   partial-score answer (length 2n - 2, not the optimal one).
// original: corpus/submissions/4801875.cpp
let n: Int = read()
domain City = 0..n
var red: [City, City]Bool = false
for i in City:
    if i == City.first: continue
    let row: Str = read()
    for j in ..i:
        red[i, j] = row[j.to_idx] == 'R'
        red[j, i] = red[i, j]

// the closed DFS walk from the first city over roads of one colour, if it
// reaches everyone
fn tour(colour: Bool) -> ?List<City>:
    var seen: [City]Bool = false
    var walk: List<City> = []
    fn go(u: City):
        seen[u] = true
        walk.push(u)
        for v in City:
            if red[u, v] == colour and not seen[v]:
                go(v)
                walk.push(u)
    go(City.first!)
    // WISH C1: `if all(seen): walk else: none`; `all`, `any`, `count` are not in the prelude.
    if sum(1 for c in City if seen[c]) == n: walk else: None

var walk = tour(false) ?? tour(true)!
walk.pop()  // closed: drop the repeated start
// WISH C3: `walk.rotate_to(walk.find(c)!)`, then `print(walk)`; today the
// rotation is an offset carried by hand and the search is a loop.
var start = 0  // where the previous city's walk started
for c in City:
    while walk[start % walk.len] != c: start += 1
    start %= walk.len
    print(walk.len)
    // one city per line: today `print(list)` writes brackets (C3)
    for i in 0..walk.len: print(walk[(start + i) % walk.len].to_idx + 1)
