// A loop over the one closure a fn value can be, whose body builds a list,
// runs that body in place and the loop's body at each push, also when the
// loop stops early (`crate::splice`, `crate::fuse` in cpl-codegen).
let (n, m): (1..=10, 1..=10) = read()
domain Row = 0..n
domain Col = 0..m
let g = read<[Row, Col]Char>()
fn find(ch: Char) -> (Row, Col) = [(r, c) for r in Row for c in Col if g[r, c] == ch][0]
print(bfs(c => c.adj4.filter(q => g[q] != '#'), from: [find('A')]).path(find('B'))?.len ?? -1)
print(bfs(c => c.adj4.filter(q => g[q] != '#'), from: [find('A')], until: find('B')).path(find('B'))?.len ?? -1)
let ys: [Col]Int = [7, 2, 8, 1, 3]
let xs = [3, 1, 4, 1, 5, 9, 2, 6]
fn upto(f: Int -> List<Int>, cap: Int) -> Int:
    var seen = 0
    for y in f(cap):
        seen += y
        if seen > cap: break
    seen
print(upto(c => [ys[j] for j in Col if ys[j] != 8], 9))
print(upto(k => [xs[i] * 2 for i in 0..k if xs[i] != 1], 8))
fn nested(f: Int -> List<Int>) -> Int:
    var t = 0
    for y in f(1):
        for z in 0..y:
            if z == 2: break
            t += 1
        if y == 3: continue
        t += 10
        if t > 40: break
    t
print(nested(c => [ys[j] + c for j in Col]))
fn nested_no_continue(f: Int -> List<Int>) -> Int:
    var t = 0
    for y in f(1):
        for z in 0..y:
            if z == 2: break
            t += 1
        t += 10
        if t > 40: break
    t
print(nested_no_continue(c => [ys[j] + c for j in Col]))
