// CCO 2020 P4, Travelling Salesperson: n cities, every pair joined by a
// red or a blue road. For each city print a walk that starts there and
// visits every city. The walks printed here never change colour: one of
// the two colours always connects all the cities (a graph or its
// complement is connected), and the Euler walk of a DFS tree in that
// colour is a closed walk through every city, which can be entered at any
// city. (This is what the accepted original prints; the statement was not
// re-read.)
// input: n, then for i = 2..n a string of i - 1 letters R/B, the roads to
// the earlier cities
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
    go(City.first)
    // WISH C1: `if all(seen): walk else: none`; `all`, `any`, `count` are not in the prelude.
    if sum(1 for c in City if seen[c]) == n: walk else: none

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
