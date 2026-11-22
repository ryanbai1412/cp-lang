// expect: ok, output "8"
domain B = 0..3
let a: [Subset<B>]Int = [Subset<B>] x => 1
print(a.sum())
