// expect: ok, output "11\n22"
// rule: 01.6.3
for (var left, v) in [(2, 10), (3, 20)]:
    left -= 1
    print(left + v)
