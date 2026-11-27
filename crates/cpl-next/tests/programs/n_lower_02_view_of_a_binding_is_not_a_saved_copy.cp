// expect: ok, output "3 4\n4 4"
// An operand that reads a binding nothing can write is passed as that binding even when a later
// operand calls a fn: std's `drop(self, k)` passes `self` and `clamp_count(k, self.len)` to
// `piece`, and the view `piece` returns views `self`, not a copy that dies when `drop` returns.
let a = [1, 2, 3, 4]
let t = a.drop(2)
print(t[2], t[3])
print(a.drop_last(1).take(10).len + 1, a.take_last(1).first!)
