// expect: ok, output "7"
// The types `Prefix.filling`'s signature names (`FillPrefix`, `FillOrder`) come with `Prefix`, read from that signature.
var f: FillPrefix<Int> = Prefix<Int>.filling(2, order: FillOrder.Forward)
f.set(0, 3)
f.set(1, 4)
print(f.fold(0..2))
