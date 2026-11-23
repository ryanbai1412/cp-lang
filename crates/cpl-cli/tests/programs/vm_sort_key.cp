let n: Int = read()
var xs: List<Int> = [i * 37 % 11 for i in 0..n]
xs.sort(by: x => x % 3)
print(xs)
xs.sort(by: x => -x, desc: n > 5)
print(xs)
var ws = ["pear", "fig", "banana", "kiwi", "plum", "apple"]
ws.sort(by: w => w.len, desc: true)
print(ws)
ws.sort(by: w => (w.len, w))
print(ws)
var ps: List<(Int, Str)> = [(3, "c"), (1, "a"), (3, "a"), (2, "b")]
ps.sort(by: p => p.0)
print(ps)
var arr: [0..5]Int = [5, 3, 9, 1, 7]
arr.sort(by: x => -x)
print(arr)
var rows: List<List<Int>> = [[3, 1], [1, 2], [2, 0]]
rows[0].sort(by: x => -x)
rows.sort(by: r => r[1])
print(rows)
var empty: List<Int> = []
empty.sort(by: x => x)
print(empty.len)
