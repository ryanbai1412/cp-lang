// expect: ok, output "55 4 2\nnone none"
use std.spec.{guess_at, berlekamp_massey}
type Z = Mod<998_244_353>
let fib = [Z(1), Z(1), Z(2), Z(3), Z(5)]
print(guess_at(fib, 9)!, guess_at([Z(1), Z(2), Z(4), Z(8)], 2)!, berlekamp_massey(fib).len)
let short = guess_at([Z(1), Z(5), Z(2), Z(7)], 10)
let one = guess_at([Z(3)], 5)
print(if short.is_some: "some" else: "none", if one.is_some: "some" else: "none")
