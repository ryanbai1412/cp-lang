// Educational DP Y, Grid 2: count the down/right paths from the top-left to
// the bottom-right corner of an h x w grid that avoid n walls.
// original: corpus/submissions/4133511.cpp

const P = 1000000007
type Z = Mod<P>

let h: 2..=1e5 = read()
let w: 2..=1e5 = read()
let n: 1..=3000 = read()
domain Wall = 0..n
let walls: [Wall](r: 1..=h, c: 1..=w) = read()

let comb = Comb<P>.new(h + w)
// paths(a, b): monotone paths from a to b; 0 when b is not below-right of a.
fn paths(a: (r: Int, c: Int), b: (r: Int, c: Int)) -> Z = comb.choose(b.r - a.r + b.c - a.c, b.r - a.r)

// clean[i]: paths from the start to stop i that touch no wall on the way:
// all paths, minus those whose first wall is an earlier stop j.
var stops = sorted(walls)
stops.push((r: h, c: w))
var clean = List.filled(stops.len, Z(0))
for i in 0..stops.len:
    clean[i] = paths((r: 1, c: 1), stops[i])
    for j in 0..i: clean[i] -= clean[j] * paths(stops[j], stops[i])
print(clean[n])
