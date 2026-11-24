// expect: ok, output "5\n4\n3\n5 4 3"
let s = [3, 1, 4, 1, 5].step(2)
for x in s.iter_rev(): print(x)
print([x for x in s.rev])
