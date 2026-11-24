// popcount of Int and U64 words: one instruction, whatever the value.
use std.math.{popcount, Word}
let n: Int = read()
var s = 0
for i in 0..64 * n:
    let x = i * 7919 % n
    s += popcount(x) // timed: math.popcount
    s += popcount(x.word()) // timed: math.Int.word
    s += popcount(U64(x).word()) // timed: math.U64.word
print(s)
