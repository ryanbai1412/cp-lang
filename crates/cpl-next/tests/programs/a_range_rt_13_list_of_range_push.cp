// expect: R0201
// rule: R03.1.4
// input: "3\n"
// Range with run-time ends as a list element type: each push is a checked store.
let n: Int = read()
var xs: List<1..=n> = List.new()
xs.push(3)
xs.push(4)  // <- here
print(xs.len)
