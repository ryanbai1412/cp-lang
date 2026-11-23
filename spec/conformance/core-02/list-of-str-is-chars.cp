// expect: ok, output "L 3"
// `List(s)` of a `Str` is a `List<Char>` in the checker as in the interpreter (round 25, F2).
let s = "LIT"
let t = List(s)
let c: Char = t[0]
print(c, t.len)
