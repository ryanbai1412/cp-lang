// expect: ok, output "1 3 2 b a"
// rule: 07.11.2
let a = [3, 1, 2]
let s = a.sorted()
let p = [(1, 'b'), (0, 'x'), (1, 'a')]
let q = p.sorted(by: t => t.0)
print(s[0], a[0], [1, 1, 2].deduped().len, q[1].1, q[2].1)
