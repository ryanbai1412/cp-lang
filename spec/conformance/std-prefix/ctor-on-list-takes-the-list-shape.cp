// expect: ok, output "7"
// `Prefix(a)` on a `List<T>` or a `[0..n]T` builds the shape whose `new` takes a `List<T>`: `Prefix<T>`, read by `Int` cuts.
let l: List<Int> = [4, 1, 5]
let p: Prefix<Int> = Prefix(l)
let xs = [3, 1, 4]
let q: Prefix<Int> = Prefix(xs)
print(p.fold(0..1) + q.cut(1))
