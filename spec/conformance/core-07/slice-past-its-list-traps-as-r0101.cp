// expect: R0101 at 5
// input: "4"
// rule: 07.4.5
// A slice that ends past its list is an index out of range, R0101, after the slice that fits has printed.
let a = [1, 2, 3]
let r: Int = read()
print(a[1..2])
print(a[1..r])
