// expect: ok, output "3 9 1"
// rule: R02.5.2 R04.2.18
// A tuple or record literal where an optional of it is wanted takes its field types, so a
// literal part is a position of the domain the field is, as it would be without the `?`.
domain V = 1..=5
let w: ?(V, Int) = (2, 9)
let r: ?(k: V, n: Int) = (k: 4, n: 1)
print(Int(w!._0) + 1, w!._1, r!.k.ord - 2)
