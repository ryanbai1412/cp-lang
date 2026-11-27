// expect: ok, output "6\n2\n-1\n6"
// rule: 01.10.1
// A generator argument to a free fn that takes no sequence is the method fold of that sequence.
let xs = [3, 1, 2]
print(max(x * 2 for x in xs)!)
print(min(x * 2 for x in xs)!)
print(min(x for x in xs[0..0]) ?? -1)
print(sum(x for x in xs))
