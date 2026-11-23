// expect: ok, output "4 2\n5 4\n2 3 3\n0"
// rule: R04a.9.3
// A step view walks its source's elements with their type, so it collects and concatenates; `odd` starts at the second.
domain V = 0..5
let p = [V(0), V(1), V(2), V(3), V(4)]
let q = p[..3].even.collect() + p[3..].collect()
print(q.len, q[1].ord)
let r = p.odd.collect() + p.even.collect()
print(r.len, r[4].ord)
print(p.odd.len, p.odd.at(3).ord, [10, 20].odd.len + 2)
print([7].odd.len)
