// expect: ok, output "AB 2 RW"
// rule: 04.4.22
// A `List<Char>` passed where a `Str` is wanted is one, and `Str(cs)` makes it one for a `Str` method; only the receiver of a `self: Str` fn takes no conversion.
let cs = ['a', 'B']
fn f(s: Str) -> Int = s.len
print(Str(cs).to_upper(), f(cs), "".join(['R', 'W']))
