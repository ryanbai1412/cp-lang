use std.seq.{Perm}
@!multitest
let n: 1..=2e5 = read()
let p: Perm<1..=n> = read()
print(p[1])
