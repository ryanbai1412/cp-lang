// expect: ok, output "0 0 0"
// rule: std.prefix.Prefix.filling
domain D = 0..3
var r1 = Prefix<D, Int>.filling(order: FillOrder.Reverse)
domain R = 0..2
domain C = 0..2
var r2 = Prefix<R, C, Int>.filling(order: FillOrder.Reverse)
var rl = Prefix<Int>.filling(3, order: FillOrder.Reverse)
print(r1.fold(D(2).r..D(2).r), r2.fold(R(1).r..R(1).r, C(1).r..C(1).r), rl.fold(3..3))
