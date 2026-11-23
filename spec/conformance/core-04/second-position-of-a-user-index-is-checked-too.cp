// expect: R0001 at 12
// input: "1 16"
// Each conversion to a position of a user `Index` checks its own ordinal: one valid `Sub.nth(a)` does not vouch for a later `Sub.nth(b)`.
struct Sub(bits: Int)

impl Index for Sub:
    let count = 1 << 4
    @get fn ord(self) -> Int = bits
    fn nth(k: Int) -> Sub = Sub(k)

let a: Int = read()
let b: Int = read()
let s = Sub.nth(a)
let u = Sub.nth(b)
print(s.bits, u.bits)
