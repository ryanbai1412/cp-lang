// expect: ok, output "6 -1\n2"
// rule: R02.4.1
// `x.map(f)` on a `?T` is `Optional`'s own member, found before `Seq`'s `map`: it keeps `None`,
// and it chains after a `find`.
let a: ?Int = Some(3)
let b: ?Int = None
print(a.map(x => x * 2) ?? -1, b.map(x => x * 2) ?? -1)
print([5, 7, 9].find(x => x > 6).map(x => x - 5)!)
