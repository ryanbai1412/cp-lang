// expect: ok, output "ab\nabc"
// rule: 01.7.1
// The binding branch's slice is a view; an owned `else` makes the `if` give an owned `Str`.
let s = "abc"
let cut: ?Int = Some(2)
let head = if let q = cut: s[..q] else: s
print(head)
let none: ?Int = None
let whole = if let q = none: s[..q] else: s
print(whole)
