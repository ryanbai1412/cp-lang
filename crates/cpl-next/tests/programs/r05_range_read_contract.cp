// expect: R0502
// input: "4\n9\n"
// A read into a range with run-time ends is input outside its contract, not a range store.
let n: Int = read()
let k = read<1..=n>() // <- here
print(k)
