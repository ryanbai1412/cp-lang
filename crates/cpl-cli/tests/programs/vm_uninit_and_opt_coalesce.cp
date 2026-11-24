// A `let` the lowering leaves without a value (a lambda taking a record's
// fields) holds its type's zero on the vm, and `a ?? b` of two optionals
// stays optional there.
let words: [3](w: Str, n: Int) = [(w: "ab", n: 2), (w: "c", n: 5), (w: "", n: 0)]
print(words.count((w, n) => w.len == n), words.map((w, n) => w + "!"))
fn later(a: ?Int, b: ?Int) -> ?Int = b ?? a
let k: Int = read()
var best: ?Int = None
for i in 0..k:
    best = later(best, if i % 3 == 1: Some(i * 10) else: None)
print(best ?? -1, later(None, None) ?? -2, later(Some(4), None)!)
