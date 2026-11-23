// expect: ok, output "120 100"
// The free `sum` takes a piece of a `.rev` like any other Seq (generator find).
let a = [10, 20, 30, 40, 50]
print(sum(a.rev.take(3)), sum(a.rev.drop(1)))
