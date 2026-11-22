// expect: ok, output "-1\n7"
// rule: 10.1.2
// input: "2\n0\n7\n"
@!multitest
let n: Int = read()
if n == 0:
    print(-1)
    return
print(n)
