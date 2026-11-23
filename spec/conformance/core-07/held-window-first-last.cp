// expect: ok, output "5 4\n1 2\n3 3\nc d"
// rule: 07.7.9
// `first`/`last` of a window read through a held `windows` value see the window, not `None`.
let a = [5, 1, 4, 2, 3]
let w = a.windows(3)
print(w[0].first!, w[0].last!)
print(w[1].first ?? -1, w[1].last ?? -1)
print(w[2].last!, w[w.len - 1].last!)
let cs = ['a', 'b', 'c', 'd']
let p = cs.windows(2)
print(p[2].first!, p[2].last!)
