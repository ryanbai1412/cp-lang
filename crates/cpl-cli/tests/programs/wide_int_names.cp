let mask: U64 = 1 << 40
let wide: I128 = I128(mask) * 3
print(mask | 5, wide)
