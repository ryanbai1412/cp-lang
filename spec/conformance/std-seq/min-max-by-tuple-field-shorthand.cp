// expect: ok, output "1 2\n3 1\n2 9"
// `.0` and `.1` as `by:` are the closures `p => p.0` and `p => p.1`.
let ps = [(3, 1), (1, 2), (2, 9)]
print(ps.min(by: .0)!)
print(ps.min(by: .1)!)
print(ps.max(by: .1)!)
