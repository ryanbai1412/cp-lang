// expect: ok, output "4 6 6 0 3\n2 2 0 2 0 YES NO\n3 2 2 2\n2 0 1 0 1\n0 1 2\n1 2 3\n2 3 0\n3 0 1\nNO 1 2 0"
domain D = 0..3
let a: [D]Int = [D] v => 1 + Int(v)
let p = Prefix<D, Int>(a)
print(p.cyclic_sum(a.ring.span(D(2), 2)), p.cyclic_sum(a.ring.span(D(0), 3)), p.cyclic_sum(a.ring.span(D(1), 3)), p.cyclic_sum(a.ring.span(D(1), 0)), p.cyclic_sum(a.ring.span(D(0), 2)))
let s = [1, 2, 3].ring.span(2, 2)
print(s.len, s.start, s.end, s.keys.collect(), s.contains(0), s.contains(1))
print([1, 2, 3].ring.windows(2).count(), [w.len for w in [1, 2, 3].ring.windows(2)])
let w = [1, 2, 3].ring.windows(2).first!
print(w.range.len, w.start, w.end, w.keys.collect())
print([[k for k in x.keys] for x in [5, 6, 7, 8].ring.windows(3)])
print([1, 2, 3].ring.span(0, 0).contains(0), [1, 2, 3].ring.span(1, 3).keys.collect())
