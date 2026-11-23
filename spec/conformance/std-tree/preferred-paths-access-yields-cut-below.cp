// expect: ok, output "2\n2 YES"
// rule: std.tree.PreferredPaths.access
// `access(v)` also yields the cut below `v` when `v` stops preferring a child, with `gained: None`.
domain V = 0..3
let t = Rooted.from_parents([V] v => if v.ord == 0: None else: Some(V(v.ord - 1)))
var pp = PreferredPaths(t)
print(pp.access(2).count())
for s in pp.access(1): print(s.lost!, yes(s.gained == None))
