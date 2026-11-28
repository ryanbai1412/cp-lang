// expect: ok, output "1"
// engines: next
print(if [1, 2] == [1, 2] and [1] != [2]: 1 else: 0)
