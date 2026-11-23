// expect: ok, output "UDLR"
// rule: 02.3.2
// input: "UDLR\n"
// `Str of "..."` is a type like any other, so it is a type argument to `read` too (rebuild round 20 IB3).
let s = read<Str of "UDLR">()
print(s)
