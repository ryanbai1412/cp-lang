// expect: ok
// rule: judge.3
// tool: build --judge big -o /dev/null
// file: cp.toml "[judge.big]\nextends = \"generic\"\nint128 = true\n"
// has: "judge `big`"
// lacks: "__int128"
// A judge whose compiler has `__int128` builds a program using `I128`.
let a: I128 = read()
print(a * a)
