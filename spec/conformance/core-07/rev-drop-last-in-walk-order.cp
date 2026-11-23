// expect: ok, output "30 20\n2 1"
// rule: 07.3.5
// `a.rev.drop_last(k)` cuts in the order `a.rev` walks: the source without its first one, still walked backwards and keeping the source's keys.
let a = [10, 20, 30]
print(List(a.rev.drop_last(1)))
print([k for (k, x) in a.rev.drop_last(1).items])
