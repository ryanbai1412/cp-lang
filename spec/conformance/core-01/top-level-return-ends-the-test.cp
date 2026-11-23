// expect: ok, output "zero\n10\nzero"
// rule: 01.6.5 10.1.2
// input: "3\n0\n5\n0\n"
// In a multitest program, top-level `return` ends the current test, not the program; `if c: s1; s2` puts both statements under the `if` (rebuild 7, H-RETURN-ONE-LINE).
@!multitest
let n: Int = read()
if n == 0: print("zero"); return
print(n * 2)
