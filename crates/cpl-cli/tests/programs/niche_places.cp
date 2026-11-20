// The inside of a `?Bool` or `?Char` taken as a place needs a `bool&` or a
// `u8&`, which the packed form has none of: this program keeps both as
// std::optional. A `?V` gives an `int&` either way, and an `Int` table that
// is written through `(x ??= v)` is not packed.
let n: 1..=20 = read()
domain V = 0..n
let s: Str = read()
var flag: ?Bool = none
(flag ??= false) = true
var ch: ?Char = none
(ch ??= 'a') = s[1]
var at: ?V = none
(at ??= V.first) = V.last
var t: [V]?Int = none
(t[V.first] ??= 1) = 42
var bytes: List<?Char> = [none]
for c in s: bytes.push(c)
print(flag ?? false, Int(ch ?? '-'), at ?? V.first, t[V.first] ?? 0, t[V.last] ?? -1, flag == none, bytes.len)
print(Int(bytes[1] ?? 'x'), Int(bytes[2] ?? 'x'), bytes[0] == none, bytes[2] == none)
