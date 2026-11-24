// expect: E0602
// rule: 05.7.6
// Moving a list while a named view of it is alive is E0602, the same error as any lent source; the view is a hold, not a closure capture (E0607).
var a = [1, 2, 3]
let v = a[0..2]
let b = a // <- here
print(v.len, b.len)
