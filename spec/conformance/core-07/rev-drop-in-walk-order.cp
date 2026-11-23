// expect: ok, output "20 10\n1 0"
// rule: 07.3.5
// `a.rev.drop(k)` cuts in the order `a.rev` walks: the source without its last one, still walked backwards and keeping the source's keys.
let a = [10, 20, 30]
print(List(a.rev.drop(1)))
print([k for (k, x) in a.rev.drop(1).items])
