// expect: ok, output "a ab xy abc 3"
// rule: 07.8.1
// `Str(c)` on a Char is its one-character word, and `Str(cs)` converts any `Seq<Char>` (decision 678).
let c = 'a'
print(Str(c), Str(['a', 'b']), Str(x for x in "xy"), "ab" + Str('c'), Str(c).len + Str("bc").len)
