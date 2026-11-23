// expect: ok, output "5"
domain D = 0..4
let fw = Fenwick<D, Int>([D] i => Int(i) + 1)
print(fw.fold(D(1)..D(3)))
