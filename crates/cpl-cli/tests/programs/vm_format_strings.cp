let n: Int = read()
let xs: [0..3]Int = [5, 6, 7]
var h: U64 = U64(1) << 63
h ^= U64(5)
h |= U64(2)
h &= ~U64(1)
for i in 0..n:
    let s = "v={i} sq={i * i} xs={xs}"
    print(s, "{h}")
