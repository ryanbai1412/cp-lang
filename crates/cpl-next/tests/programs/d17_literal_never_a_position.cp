// expect: E0902
// input: "3\n"
// An integer literal that the types of a domain's ends put outside it is a compile error
// (R04.2.18), below the start as much as past the end.
domain V = 2..=read<2..=5>()
var a: [V]Int = 0
print(a[5])
print(a[1]) // <- here
