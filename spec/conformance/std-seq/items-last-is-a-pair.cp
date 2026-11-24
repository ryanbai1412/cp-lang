// expect: ok, output "2 9"
// rule: std.seq.last
// `last` of `ys.items` is the `(key, value)` pair its walk yields, not the source's element.
let ys: List<Int> = [4, 7, 9]
print(ys.items.last!)
