// expect: ok, output "2 3\n3\n1 7"
// Moving `a.xs` into a struct leaves `a`'s other fields readable, and assigning the moved field
// makes `a` whole again.
struct Box:
    xs: List<Int>
struct Pair:
    var xs: List<Int>
    n: Int
var a = Pair([1, 2], 3)
let b = Box(a.xs)
print(b.xs.len, a.n)
print(a.n)
a.xs = [7]
let c = a
print(c.xs.len, c.xs[0])
