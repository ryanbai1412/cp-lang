// expect: R0202 at 4 (checked)
// input: "-9000000000000000000 9000000000000000000\n"
// An `Int` range keeps its length check: this span does not fit in 64 bits.
let lo: Int = read()
let hi: Int = read()
print((lo..hi).len)
