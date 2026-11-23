// A shift whose declared range stays below 63 fits in `Int`.
let k: 0..=62 = read()
print(1 << k)
print(3 << (k / 2))
