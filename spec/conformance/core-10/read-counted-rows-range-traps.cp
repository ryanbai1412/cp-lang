// expect: R0502 at 5 (checked)
// rule: 10.2.3
// input: "2 1 12\n1 3\n"
// A count-prefixed row checks each element against its range, as `[n]1..=9` does.
let rows = read<[2]List<1..=9>>()
print(rows)
