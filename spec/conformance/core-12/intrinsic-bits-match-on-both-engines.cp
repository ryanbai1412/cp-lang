// expect: ok, output "3 60 4 16 64 60 64 1 4"
// rule: 12.14.3
// input: "8\n"
// std's bit fns are `@intrinsic` bodies: an `Int` counts its two's complement word, a `U64` its own, and 0 gives 64 zero bits, on both engines.
let n: Int = read()
print(ctz(n), clz(n), bit_width(n), bit_ceil(n + 1), ctz(0 * n), clz(U64(n)), bit_width(-1), bit_ceil(-n), ctz(U64(n) * U64(2)))
