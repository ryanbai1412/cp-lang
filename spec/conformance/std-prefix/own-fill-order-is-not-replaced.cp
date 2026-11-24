// expect: ok, output "1"
// A program's own `FillOrder` stays its own beside `Prefix`.
enum FillOrder = Up | Down
let p = Prefix<Int>([1, 2])
let o = FillOrder.Up
print(if o == FillOrder.Up: p.cut(1) else: 0)
