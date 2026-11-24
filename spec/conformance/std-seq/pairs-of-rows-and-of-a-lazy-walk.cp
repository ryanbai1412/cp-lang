// expect: ok, output "3 1 2 3 4\n2 3\n2 4\n3 4\n2"
// `pairs` over rows reads each row where it is; over a lazy walk it walks once.
let rows: [0..3][0..2]Int = [[1, 2], [3, 4], [5, 6]]
var seen = 0
var first: List<List<Int>> = []
for (a, b) in rows.pairs():
    seen += 1
    if seen == 1: first = [a, b]
print(seen, first[0], first[1])
print([1, 2, 3, 4].filter(x => x > 1).pairs().collect())
fn evens(n: Int) -> yields Int:
    for i in 0..n if i % 2 == 0: yield i
print(List(evens(4)).len)
