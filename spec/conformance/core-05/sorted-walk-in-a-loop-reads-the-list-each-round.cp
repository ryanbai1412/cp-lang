// expect: ok, output "126 5\n2 3\n4 3\n9 3\n1 2 3 3 1 2"
// rule: 05.3.1
// A `for` over `xs.sorted()` inside a loop reads `xs` again each round, a closure that reads the list keeps it, and a sort in one branch leaves the source as it was.
var edges = [(3, 1), (1, 2), (2, 0)]
var total = 0
for round in 0..2:
    for e in edges.sorted(by: e => e.0):
        total += e.0 * 10 + e.1
    edges.push((round, round))
print(total, edges.len)
let keep = [4, 2, 9]
let peek = () => keep.len
for x in keep.sorted():
    print(x, peek())
let last = [3, 1, 2]
let early = if last.len > 2: last.sorted() else: [0]
print(early, last)
