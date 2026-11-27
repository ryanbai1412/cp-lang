// expect: ok, output "ab 2\nx y z\nrest of it\n3"
// input: "  ab  \r\n x\ty z \nrest of it\n3\n"
// rule: R10.2.1
// `Str` and `Char` are ordinary `Read` impls over the input's bytes: a `Str` is one token, a `Char` takes the whitespace before it, and a `read_line()` after a line's last token reads the next line.
let s: Str = read()
print(s, s.len)
let a: Char = read()
let b = read<Char>()
let t = read<Str>()
print(a, b, t)
print(read_line())
print(read<Int>())
