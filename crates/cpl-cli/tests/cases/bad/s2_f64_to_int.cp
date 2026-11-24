// error: `n` is declared an Int, but `x` is an F64
// Synthesis #2: this spelling stays rejected (research/archive/architecture/language-architecture-synthesis-2.md, rejected list).
let x = 2.5
let n: Int = x
print(n)
