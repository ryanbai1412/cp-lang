// expect: ok, output "Bob\nAlice"
// rule: 10.1.1 09.1.1
// input: "2\n1\n2\n"
// A bare `with io(...)` above `@!multitest` covers every case; the divider does not start a block (rebuild 8, H-WITH-ACROSS-MULTITEST).
with io(answers: ("Alice", "Bob"))
@!multitest
let n: Int = read()
print(yes(n > 1))
