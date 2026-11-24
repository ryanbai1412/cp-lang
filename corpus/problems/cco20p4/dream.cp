// CCO 2020 P4 Travelling Salesperson: every pair of cities is joined by a red or blue road.
// For each start, give a route over every city that changes colour at most once.
// Grow the route one city at a time, kept as two runs meeting at a junction: `near` from the
// start in one colour, then `far` in the other. A new city always fits next to the junction,
// and when the first run would be empty the whole route flips to the other colour.
let n: 2..=2000 = read()
domain City = 0..n
var red: [City, City]Bool = false
for i in City:
    if i == City.first: continue
    let row: Str = read()
    for j in ..i:
        red[i, j] = row[j.ord] == 'R'
        red[j, i] = red[i, j]

fn route(s: City) -> List<City>:
    var x = false               // the colour of the `tail` run's roads: red when true
    var tail: List<City> = []   // in route order, ending at the junction
    var head: List<City> = [s]  // reversed: s first, the junction last
    for u in City:
        if u == s: continue
        if tail.len > 0 and
                head.len == 1 and
                red[u, tail.last!] != x and
                red[u, s] == x and
                red[tail.last!, s] == x:
            while let c = tail.pop(): head.push(c)
            x = not x
        if tail.is_empty:
            if red[u, head.last!] != x: head.push(u) else: tail.push(u)
            continue
        let (a, b) = (tail.last!, head.last!)
        if red[u, a] == x: tail.push(u)
        elif red[u, b] != x:
            tail.pop()
            head += [u, a]
        elif red[a, b] != x:
            tail.pop()
            head += [a, u]
        else:
            head.pop()
            tail += [b, u]
    head + tail.rev.collect()

for s in City:
    let r = route(s)
    print(r.len)
    print(r.map(c => c.ord + 1), sep: "\n")
