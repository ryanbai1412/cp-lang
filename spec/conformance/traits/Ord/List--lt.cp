// expect: ok, output "1"
// engines: next
print(if [1, 2] < [1, 3] and [2] > [1, 9]: 1 else: 0)
