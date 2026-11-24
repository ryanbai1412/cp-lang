// expect: ok, output "7 97 1 0 2 2147483650 2"
// rule: 04.2.1
// `hash_token` on each built-in element type runs its std `Hashable` impl.
use std.strings.{HashVal, Hashable}
domain V = 0..4
fn tok<T: Hashable>(x: T) -> Int = x.hash_token
print(tok(7), tok('a'), tok(true), tok(false), tok(V(2)), tok(HashVal(1, 2)), 2.hash_token + V(0).hash_token)
