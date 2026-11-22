// A bare `pop` as a statement, `scan` over an array, a `Graph` from a list
// of pairs built as the program runs, and `count` of a value.
var xs = [3, 1, 2]
xs.pop()
var s = Set<Int>()
s.insert(4)
s.insert(5)
s.pop_first()
var q = Deque<Int>()
q.push(1)
q.pop()
print(xs, s, q.len)
let a: [0..3]Int = [1, 2, 3]
print(scan(a, (x, y) => x + y))
domain V = 0..4
var es: List<(V, V)> = []
es.push((V(0), V(1)))
es.push((V(1), V(2)))
let g = Graph.directed(es)
let h = Graph.undirected(es)
for u in V: print(g[u], h[u])
let word: Str = read()
print(word.count('a'), "".count('a'), [1, 2, 1].count(1), a.count(3), "aXa".count('x'))
