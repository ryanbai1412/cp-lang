// expect: ok, output "1 a b\n2 a b\n3"
// rule: 01.1.4
// CRLF source: `\r\n` ends a line and never enters a token, comment or string.
let xs = [1, 2]
for x in xs:
    print(x, "a b")  // trailing comment
let s = "x"
print(s.len + 2)
