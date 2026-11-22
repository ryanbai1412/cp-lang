// expect: ok, output "1"
// rule: 06.5.1
var shared = 0
shared += 1
print(shared)
@!multitest(until_eof)
print(9)
