// expect: ok, output "7 7"
// rule: 02.4.7
// The extent of `?` is the statement right-hand side: `v` is still a `?Int`.
let o: ?Int = None
let v = o?
let w = o? + 1
print(v ?? 7, w ?? 7)
