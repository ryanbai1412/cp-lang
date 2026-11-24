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
let n: 2..=2000 = read()
domain City = 0..n
var red: [City, City]Bool = false
for i in City:
    if i == City.first: continue
    let row: Str = read()
    for j in ..i:
        red[i, j] = row[j.ord] == 'R'
        red[j, i] = red[i, j]

for s in City:
    var x = false  // colour of the first part's roads: red when true
    var first: List<City> = []  // in path order; its last city is a, at the junction
    var second: List<City> = [s]  // back to front: s stays first, the last city is b
    for u in City:
        if u == s: continue
        if first.len > 0 and second.len == 1 and red[u, first.last!] != x and red[u, s] == x and red[first.last!, s] == x:
            // the whole path is colour x: it all becomes the second part
            while let c = first.pop(): second.push(c)
            x = not x
        if first.len == 0:
            if red[u, second.last!] != x: second.push(u) else: first.push(u)
            continue
        let (a, b) = (first.last!, second.last!)
        if red[u, a] == x: first.push(u)
        elif red[u, b] != x:  // a, u, b in the second part
            first.pop()
            second.push(u)
            second.push(a)
        elif red[a, b] != x:  // u, a, b in the second part
            first.pop()
            second.push(a)
            second.push(u)
        else:  // a, b, u in the first part
            second.pop()
            first.push(b)
            first.push(u)
    print(n)
    for c in second: print(c.ord + 1)
    while let c = first.pop(): print(c.ord + 1)
