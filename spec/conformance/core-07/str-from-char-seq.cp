// expect: ok, output "ac\nac\na-c"
// rule: 07.8.1
// A `Seq<Char>` becomes a `Str` by constructor or `.collect()`; `join` takes `Str` parts (rebuild round 6 H-STR-FROM-CHARS).
let s = "abc"
print(Str(s.filter(c => c != ('b'))))
let w: Str = s.filter(c => c != ('b')).collect()
print(w)
print("-".join(["a", "c"]))
