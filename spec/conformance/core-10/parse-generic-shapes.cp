// expect: ok, output "5 b 6 0 3"
// input: "3 1\n"
// rule: 10.2.2 10.2.4
// std's `parse` reads a user struct field by field, keeps a fixed array length, and reads counted positions in the caller's base.
with io(base: 1)
struct P:
    x: Int
    name: Str
let q = "5 b".parse<P>()!
let a = "1 2 3".parse<[3]Int>()!
let n = read<Int>()
domain E = [n]
let e = read<E>()
print(q.x, q.name, a.sum(), Int(e), n)
