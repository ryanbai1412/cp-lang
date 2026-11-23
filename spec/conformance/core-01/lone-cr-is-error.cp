// expect: E0107 at 2
// rule: 01.1.4
// A `\r` that does not start a `\r\n` is not a line end, so it cannot split two statements.
let x = 1print(x)
