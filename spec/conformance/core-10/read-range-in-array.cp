// expect: R0502 (checked)
// rule: 10.2.2 08.4.5
// input: "1 3\n2 0\n"
// A `[2]1..=3` reads each cell by the range type's own `Read`: a cell outside it is `None` from
// a string and traps from standard input.
assert read<[2]1..=3>(from: "1 2") != None
assert read<[2]1..=3>(from: "1 9") == None
let a = read<[2]1..=3>()
assert a[0] == 1 and a[1] == 3
let b = read<[2]1..=3>() // <- here
print(b)
