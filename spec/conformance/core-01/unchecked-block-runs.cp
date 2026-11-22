// expect: ok, output "3"
// rule: 01.3.7
var s = 0
unchecked:
    s = 1 + 2
print(s)
