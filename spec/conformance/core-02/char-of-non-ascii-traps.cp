// expect: R0201 (checked)
// rule: 02.2.1
// input: "194\n"
// `Char` is ASCII 0..=127, so `Char(194)` traps instead of printing a raw byte (rebuild round 15).
let i = read<Int>()
print(Char(i)) // <- here
