// expect: ok, output "40 30\n50 40\n120 50 10"
// A piece of a `.rev` and a `.rev` of a piece walk backward from their own ends.
let a = [10, 20, 30, 40, 50]
print(List(a.rev.drop(1).take(2)))
print(List(a.drop(1).rev.take(2)))
print(a.rev.take(3).sum(), a.rev.first!, a.rev.last!)
