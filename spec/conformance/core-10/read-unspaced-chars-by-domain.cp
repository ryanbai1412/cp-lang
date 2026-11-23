// expect: ok, output "0 1"
// input: "101100\n"
// rule: 10.2.3
// `[I]Char` reads one character per position, with no separators needed.
domain I = 1..=6
let s = read<[I]Char>()
print(s[I.last!], s[I.first!])
