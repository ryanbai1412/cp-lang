// expect: E0302
// engines: next
// rule: 07.8.1
// A number's text is its `Show`, `"{x}"`; `Str(x)` converts only a Char or a `Seq<Char>` (decisions 390, 678).
let x = 5
print(Str(x)) // <- here
