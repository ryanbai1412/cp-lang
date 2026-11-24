// Source: CCO 2020 P4, Travelling Salesperson, https://dmoj.ca/problem/cco20p4
// limits: 7000 ms, 1024 MB
// Problem: 2 <= n <= 2000 buildings, every pair joined by a red or a blue road (line i gives
//   the colours to buildings 1..i-1). For each start, print a route of minimum length
//   (buildings visited, repeats counted) that visits every building and switches road colour
//   at most once.
// Solution: the minimum is always n: keep a path ending at the start, a first part in one
//   colour x then a second part in the other, and add the other buildings one by one next to
//   the junction (a, b): after a if u-a is x; else before b, moving a or b across the
//   junction so both new roads match their part. When the path is all x, recolour it as the
//   second part. O(1) amortized per building, O(n^2) in all; print the path reversed.
// original: corpus/submissions/6416816.cpp
// Bounds: n: 2..=2000
let n: 2..=2000 = read()
domain City = 0..n
enum Colour:
    Red
    Blue

var road: [City, City]Colour = Red
for i in City if i != City.first:
    let row: Str = read()
    for j in ..i:
        road[i, j] = if row[j.ord] == 'R': Red else: Blue
        road[j, i] = road[i, j]

for s in City:
    var x = Red  // the first part's colour
    var first: List<City> = []  // in path order; its last city is a, at the junction
    var second: List<City> = [s]  // back to front: s stays first, the last city is b
    for u in City if u != s:
        if let a = first.last:
            if second.len == 1 and road[u, a] != x and road[u, s] == x and road[a, s] == x:
                // the whole path is colour x: it all becomes the second part
                while let c = first.pop(): second.push(c)
                x = if x == Red: Blue else: Red
        let b = second.last!
        if let a = first.last:
            if road[u, a] == x: first.push(u)
            elif road[u, b] != x:  // a, u, b in the second part
                first.pop()
                second.push(u)
                second.push(a)
            elif road[a, b] != x:  // u, a, b in the second part
                first.pop()
                second.push(a)
                second.push(u)
            else:  // a, b, u in the first part
                second.pop()
                first.push(b)
                first.push(u)
        elif road[u, b] != x: second.push(u)
        else: first.push(u)
    print(n)
    for c in second: print(c.ord + 1)
    for c in first.rev: print(c.ord + 1)
