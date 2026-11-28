// expect: R0502 (checked)
// rule: 10.2.2 08.4.5
// input: "2 1 3\n2 1 9\n"
// A `List<1..=3>` reads each item by the range type's own `Read`: an item outside it is `None`
// from a string and traps from standard input.
assert read<List<1..=3>>(from: "2 1 3") == Some([1, 3])
assert read<List<1..=3>>(from: "2 1 9") == None
assert read<List<1..=3>>() == [1, 3]
let ys = read<List<1..=3>>() // <- here
print(ys)
