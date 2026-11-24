// expect: R0202
// rule: 12.14.3
// input: "8\n"
// `bit_ceil` traps where no `Int` power of two is that large, as its C++ form does.
let n: Int = read()
print(bit_ceil((n << 59) + 1)) // <- here
