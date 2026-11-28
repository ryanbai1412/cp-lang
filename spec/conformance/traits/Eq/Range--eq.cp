// expect: ok, output "1"
// engines: next
print(if (1..3) == (1..3) and (1..3) != (1..4): 1 else: 0)
