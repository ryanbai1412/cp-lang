// expect: R0201
// input: "5\n3\n"
// A range whose ends are run-time values: the ends are worked out once, where the range is
// written, and every store into the slot is checked against them.
let n: Int = read()
var i: 1..=n = read()
var xs: List<0..n> = List.new()
xs.push(i)
let f = () => xs.get(0) + i
print(f())
i = i + 2
print(i)
i += 1 // <- here
