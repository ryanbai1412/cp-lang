// error: a map literal is
// Synthesis #2: this spelling stays rejected (research/language-architecture-synthesis-2.md, rejected list).
domain D = 0..5
let a: [D]Int = [1: 7, ..0]
print(a[D(1)])
