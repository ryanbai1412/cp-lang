// expect: ok, output "01?"
// rule: 02.3.2
// input: "01?\n"
// `Str of "01?"` is a type like any other, so it is a type argument too: `read<Str of "01?">()` reads it as `let s: Str of "01?" = read()` does (rebuild round 30 F03).
let s = read<Str of "01?">()
print(s)
