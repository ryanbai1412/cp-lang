// expect: R0101 at 6
// input: "5"
// rule: 07.4.5
// A slice of a slice takes the outer slice's keys (2..4 here) and traps past them, never reading the parent past the view.
let a = [1, 2, 3, 4, 5]
let r: Int = read()
let w = a[2..4]
print(w[2..4])
print(w[2..r])
