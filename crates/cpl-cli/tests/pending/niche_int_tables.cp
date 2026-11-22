// `?Int` tables, packed with -2^63, which is not an `Int`, as `none`: each
// must read back `none`, 0, -1, Int.MIN and Int.MAX exactly as the
// interpreter does.
let n: 1..=20 = read()
domain V = 0..n

var safe: [V]?Int = none
var wild: [V]?Int = none
var grid: [V, V]?Int = none
var lst: List<?Int> = []
safe[V.first] = 0
wild[V.first] = Int.MIN
wild[V.last] = Int.MAX
for v in V:
    lst.push(none)
    if v.idx % 2 == 1: lst.push(v.idx - 1)
    if let w = v + 1:
        if let d = safe[v]:
            safe[w] max= d + 3
            safe[w] min= d + 1
        grid[v, w] = v.idx - 0
        grid[w, v] = Int.MAX
        if v.idx % 2 == 0: wild[w] = -1
for v in V:
    print(safe[v] ?? -1, safe[v] == none, wild[v] ?? 0, wild[v] == none, (wild[v] ?? 5) == Int.MIN, grid[v, V.first] ?? -7, grid[V.last, v] ?? -7)
print(lst.len, lst[0] ?? -1, lst[2] ?? -1, lst[lst.len - 1] ?? -1)
var hi: ?Int = none
var lo: ?Int = none
for v in V:
    hi max= wild[v]
    lo min= wild[v]
    hi max= safe[v]
print(hi ?? 0, lo ?? 0, hi == Int.MAX, lo == Int.MIN)
var top = Int.MIN
var bottom = Int.MAX
for v in V:
    top max= safe[v]
    bottom min= safe[v]
    bottom min= wild[v]
print(top, bottom)
