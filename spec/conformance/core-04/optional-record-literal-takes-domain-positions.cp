// expect: ok, output "2 9"
// rule: 04.2.10
// A record literal given where an optional record is declared takes the record's field types, so `2` is a position of `V`, as it would be without the `?`.
domain V = 1..=5
let w: ?(k: V, n: Int) = (k: 2, n: 9)
print(w!.k, w!.n)
