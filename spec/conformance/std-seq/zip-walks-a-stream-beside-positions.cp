// expect: ok, output "1 2 10\n2 3 20\n3 1 30\n2"
// `zip(a, b)` walks any Seq `a`, a stream too, beside `b` read by position, stopping at the shorter (R04b.1.3).
let posts = [1, 2, 3]
let costs = [10, 20, 30, 40]
for ((a, b), c) in zip(posts.cyclic_pairs(), costs): print(a, b, c)
print(zip(costs, [7, 8]).count())
