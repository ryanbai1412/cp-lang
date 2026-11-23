// expect: E0107 at 2
// rule: 01.1.4
// A lone `\r` inside a comment is an error too: it is only ever half of a `\r\n` line end (02e44dcc1).
let x = 1 // oneprint(x)
print(x)
