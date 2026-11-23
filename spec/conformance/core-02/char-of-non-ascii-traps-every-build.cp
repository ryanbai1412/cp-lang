// expect: R0201 at 4
// rule: 02.2.1
// input: "194\n"
// `Char(i)` past 127 traps in every build and both engines, not only checked runs, so a computed byte can't print raw; the first two are in range (bugcatch round 18).
let i = read<Int>()
print(Char(i - 100), Char(i - 67))
print(Char(i))
