// expect: ok, output "2 7 1 1\n2 b"
// input: "7 1 9 1 1 a 2 b\n"
// rule: 08.1.2
// `read_all` is a std body generic over `T: Read`: a ranged `Int` keeps its range and a struct reads field by field.
use std.io.{read_all}
struct P:
    x: Int
    name: Str
let xs = read_all<0..=9>(until: x => x == 9)
print(xs.len, xs[0], xs[1], read<0..=9>())
let ps = read_all<P>()
print(ps[1].x, ps[1].name)
