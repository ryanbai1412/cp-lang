// error: `a` is indexed by Pkg, but `d` is a Dep
// Synthesis #2: this spelling stays rejected (research/archive/architecture/language-architecture-synthesis-2.md, rejected list).
let n: 2..=9 = read()
domain Pkg = 0..n
domain Dep = 1..n
var a: [Pkg]Int = 0
for d in Dep: a[d] = 1
print(sum(a))
