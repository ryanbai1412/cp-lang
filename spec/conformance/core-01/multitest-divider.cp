// expect: ok, output "1"
// rule: 01.7.5
print(1)
@!multitest(until_eof)
print(2)
