// expect: ok, output "ok"
// rule: lib/random-stress gen(range), gen<[V]range>(), gen<Str of "...">(len:), gen<Graph<..>>(simple:, connected:), gen<Rooted<V>>(shape:), gen<Perm<V>>()
let n = gen(3..=8)
assert 3 <= n and n <= 8
domain V = 1..=n
let a = gen<[V]1..=1e9>()
assert a.len == n
for v in V:
    assert 1 <= a[v] and a[v] <= 1000000000

let s = gen<Str of "ab">(len: 1..=10)
assert 1 <= s.len and s.len <= 10
for c in s:
    assert c == 'a' or c == 'b'

let g = gen<Graph<V, (to: V, w: 1..=9)>>(simple: true, connected: true)
assert g.edges.len >= n - 1
for e in g.edges:
    assert e.u != e.v
    assert 1 <= e.w and e.w <= 9

let tr = gen<Rooted<V>>(shape: random)
let tp = gen<Rooted<V>>(shape: path)
let ts = gen<Rooted<V>>(shape: star)
let tc = gen<Rooted<V>>(shape: caterpillar)
let tb = gen<Rooted<V>>(shape: binary)
assert tr.depth(tr.root) == 0
assert tp.depth(V.nth(n - 1)) == n - 1
assert ts.depth(V.nth(n - 1)) == 1
assert tc.depth(tc.root) == 0
assert tb.depth(tb.root) == 0

let p = gen<Perm<V>>()
assert p.len == n
var seen: [V]Bool = [V] _ => false
for v in V:
    seen[p[v]] = true
for v in V:
    assert seen[v]
print("ok")
