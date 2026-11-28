// expect: ok, output "3"
// rule: 12.13.3
// `@stress(kind)` above a type alias is for `cpl stress`; `cpl run` ignores it.
@stress(perm)
type P = List<Int>
let p: P = [2, 0, 1]
print(p.len)
