// expect: ok, output "1 3"
// `[1..=n]` is one element, a range; `List(1..=n)` is the list (acceptance run 6, row 134).
let n = 3
print([1..=n].len, List(1..=n).len)
