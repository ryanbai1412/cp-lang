// expect: ok, output "3 2 x 1 5"
// rule: 10.2.1
// input: "3 2 1 x 4 1 5\n"
// A struct and a record with no `Read` of their own read their fields in order, whatever each
// field's type, a nested record included; interpreter and C++ agree.
struct P:
    a: Int
    b: [2]?Int
    c: (s: Str, t: 0..5)
let p = read<P>()
let r = read<(u: 0..5, v: Int)>()
print(p.a, p.b[0] ?? 9, p.c.s, r.u, r.v)
