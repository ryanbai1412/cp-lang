// expect: ok, output "1 1 2\n1 -1 -1\n-1 -1 1\n2 3 3 3 2\n1 2 4\n1 1 -1 3 1 3\n-1 -1 -1 1 -1 -1 3 -1 -1 -1\n0 2 4\n1 1"
// A list receiver of a `self: [D]T` fn is `D = Int`: each result is a list as long as it.
let xs = [3, 1, 2]
print([3, 1, 2].lis_ends())
print(xs.next_smaller().map(o => o ?? -1))
print(xs.prev_smaller().map(o => o ?? -1))
let ys = [4, 1, 3, 2, 5]
let mn = ys.ring.minimal(state: track(0, (mut s, x) => s += x, (mut s, x) => s -= x), ok: w => w.state >= 6)
print(mn.map(o => if let s = o: s.len else: -1))
let zs = [3, 1, 4, 1, 5]
print(zs.lis())
let ct = zs.cartesian_tree()
print(ct.root!, ct.parent.map(o => if let p = o: p else: -1))
print(zs.prev_same().map(o => o ?? -1), zs.next_same().map(o => o ?? -1))
print(zs.record_highs().collect())
print(zs.valleys().map(r => r.len).collect())
