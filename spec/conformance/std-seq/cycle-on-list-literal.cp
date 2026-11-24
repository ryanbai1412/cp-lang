// expect: ok, output "0 1 2\n1 2 0"
print([1, 2, 0].cycle(from: 0).collect())
print([1, 2, 0].cycle(from: 1).collect())
