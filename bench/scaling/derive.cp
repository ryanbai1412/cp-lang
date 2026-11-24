// fixed: fields
// stdin: 12
use std.derive.{eq_fields, lt_fields, show_fields, read_fields, hash_fields, gen_fields}
struct P(a: Int, b: Int, c: Int)
struct Q(a: 1..=9, b: 1..=9, c: 1..=9)
let n: Int = read()
let reps = 1 << 18
let xs = [P(i, (i * 7) % n, i % 3) for i in 0..n]
var s = 0
var h = U64(0)
for i in 0..reps:
    if eq_fields(xs[i % n], xs[(i * 7) % n]): s += 1 // timed: derive.eq_fields
    if lt_fields(xs[i % n], xs[(i * 7) % n]): s += 1 // timed: derive.lt_fields
    h ^= hash_fields(xs[i % n]) // timed: derive.hash_fields
    h ^= i.hash() // timed: derive.Int.hash
    h ^= U64(i).hash() // timed: derive.U64.hash
    h ^= I128(i).hash() // timed: derive.I128.hash
    h ^= Char(97 + i % 26).hash() // timed: derive.Char.hash
    h ^= (i % 3 == 0).hash() // timed: derive.Bool.hash
    s += gen_fields<Q>().b // timed: derive.gen_fields
for i in 0..reps / 4:
    s += show_fields(xs[i % n]).len // timed: derive.show_fields
for _ in 0..4 * n:
    s += read_fields<P>().b // timed: derive.read_fields
let str = "ab".repeat(n / 2)
for _ in 0..200:
    h ^= str.hash() // timed: derive.Str.hash [let: length = n]
print(s, h)
