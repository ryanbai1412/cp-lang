// expect: ok, output "a\nb\nb"
// rule: R10.1.2
// input: "3\n1\n2\n2\n"
// R10.1.2: top-level return inside a nested fn value call does not end the case; return in an if does.
@!multitest
let x: Int = read()
if x == 1:
    print("a")
    return
print("b")
