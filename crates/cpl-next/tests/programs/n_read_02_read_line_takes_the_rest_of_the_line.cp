// expect: ok, output "2 +3\n5 hello\n7\n3   c\n0 \n4 last"
// rule: R10.2.3
// input: "2 +3\r\nhello\r\n7  c\n\nlast\n"
// `read_line()` is an ordinary core fn over the input's bytes: after a line's last token it reads the next line, mid-line it reads the rest, and it drops a trailing `\r`.
let a: Int = read()
let b: Int = read()
print(a, "+{b}")
let l = read_line()
print(l.len, l)
let w: Int = read()
let r = read_line()
let e = read_line()
let v = read_line()
print(w)
print(r.len, r)
print(e.len, e)
print(v.len, v)
