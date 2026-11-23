// expect: ok, output "abc"
// input: "3\nabc\n"
// rule: 02.3.2
// `Str(len: n..=n)` checks an exact runtime length on read.
let n = read<Int>()
let s = read<Str(len: n..=n)>()
print(s)
