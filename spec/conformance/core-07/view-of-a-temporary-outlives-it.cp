// expect: E0603 at 2
// rule: 07.5.10 05.7.8
// An unnamed temporary lives to the end of its statement (R07.5.10), so a `let` keeping a view of it outlives its source. Name the source, or copy with `List(...)`.
let w = [3, 1, 2].sorted()[0..1]
print(w.sum())
