// expect: ok, output "c a\n0 127\nYES NO"
// rule: 02.2.1
// `Char` and `Bool` are bounded, so `max` and `min` fold them with the far end as identity:
// `Char.MIN` is `Char(0)`, `Char.MAX` is `Char(127)`, `Bool.MIN` is `false`.
let t = "abca"
print(t.fold(max), t.fold(min))
let e = ""
print(Int(e.fold(max)), Int(e.fold(min)))
print(yes([false, true].fold(max)), yes([true, false].fold(min)))
