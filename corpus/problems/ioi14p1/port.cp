// Source: IOI 2014 P1 (Rail), https://dmoj.ca/problem/ioi14p1
// Problem: interactive (function task). n stations sit in distinct blocks of a rail line of at
//   most 1e6 blocks, each type C or D; station 0 is type C at block `first`. getDistance(i, j)
//   gives the shortest route length; recover every station's block and type within each
//   subtask's limit on getDistance calls.
// Solution: query 0 against every station and sort by distance; the nearest is the D just
//   right of station 0. In that order, keep the leftmost C (l) and rightmost D (r): two
//   queries give the block if the station is a D right of l (loc[l] + dl) or a C left of r
//   (loc[r] - dr), and the turning block between them decides: a placed station there by its
//   type, else D iff it lies right of the rightmost C so far. 3n - 5 queries, O(n log n).
// Stdin: n, then hidden loc[0..n] and kind[0..n] (1 = C, 2 = D), every C with a D to its
//   right and every D with a C to its left. Simulates getDistance in O(1) from the nearest D
//   right of and the nearest C left of each block, runs find_location, and prints the
//   recovered loc[i] and kind[i]. The judge form, `@grader fn findLocation(n, first, location,
//   stype)` with `extern fn getDistance`, is blocked: a grader fn can't fill the judge's
//   arrays (E0501).
// original: corpus/problems/ioi14p1/orig.cpp (submission 4691718 plus the sample grader)
// original: corpus/submissions/4691718.cpp

domain Block = 0..1_000_000

fn find_location(n: Int, first: Block, dist: (Int, Int) -> Int) -> (loc: [n]Int, kind: [n]1..=2):
    var loc: [n]Int = first.ord
    var kind: [n]1..=2 = 1
    if n == 1: return (loc:, kind:)
    let d0 = [n] i => if i == 0: 0 else: dist(0, i)
    let ord = (1..n).sorted(by: i => d0[i])
    // placed[x]: the kind of the station found at block x.
    var placed: [Block]?(1..=2) = None
    let s1 = ord[0]
    loc[s1] = first.ord + d0[s1]
    kind[s1] = 2
    placed[first] = 1
    placed[Block(loc[s1])] = 2
    var (l, r) = (0, s1)
    var open_r = first.ord
    for a in ord.drop(1):
        let dl = dist(l, a)
        let dr = dist(r, a)
        let mid = Block((loc[r] + loc[l] + dl - dr) / 2)
        let is_d = if let k = placed[mid]: k == 1 else: mid.ord > open_r
        if is_d:
            loc[a] = loc[l] + dl
            kind[a] = 2
            if loc[a] > loc[r]: r = a
        else:
            loc[a] = loc[r] - dr
            kind[a] = 1
            if loc[a] < loc[l]: l = a
            open_r max= loc[a]
        placed[Block(loc[a])] = kind[a]
    (loc:, kind:)

let n: 1..=5000 = read()
domain S = 0..n
let at: [S]Block = read()
let real_kind: [S]1..=2 = read()

// right_d[x]: the nearest D block after x; left_c[x]: the nearest C block before x.
var has_d: [Block]Bool = false
var has_c: [Block]Bool = false
for i in S:
    if real_kind[i] == 2: has_d[at[i]] = true
    else: has_c[at[i]] = true
var right_d: [Block]?Block = None
var left_c: [Block]?Block = None
for x in Block.rev:
    if let y = x.next: right_d[x] = if has_d[y]: y else: right_d[y]
for x in Block:
    if let y = x.prev: left_c[x] = if has_c[y]: y else: left_c[y]

fn get_distance(i: Int, j: Int) -> Int:
    if i == j: return 0
    let (a, b) = (at[S(i)], at[S(j)])
    match (real_kind[S(i)], real_kind[S(j)]):
        (1, 1) => 2 * right_d[max(a, b)]!.ord - a.ord - b.ord
        (2, 2) => a.ord + b.ord - 2 * left_c[min(a, b)]!.ord
        (1, _) => if a < b: b.ord - a.ord else: 2 * (right_d[a]!.ord - left_c[b]!.ord) - a.ord + b.ord
        _ => if b < a: a.ord - b.ord else: 2 * (right_d[b]!.ord - left_c[a]!.ord) + a.ord - b.ord

let found = find_location(n, at[S(0)], get_distance)
for i in 0..n: print(found.loc[i], found.kind[i])
