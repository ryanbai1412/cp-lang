// expect: E0608
// A struct field keeps its argument (R05.5), so building `Box(a.xs)` moves the path `a.xs`
// out of `a`: `a` as a whole is gone after it.
struct Box:
    xs: List<Int>
struct Pair:
    xs: List<Int>
    n: Int
let a = Pair([1, 2], 3)
let b = Box(a.xs)
print(b.xs.len)
let c = a // <- here
