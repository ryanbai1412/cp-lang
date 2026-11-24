// expect: ok
// tool: build -o -
// lacks: "std::string before"
// A scan over a heap type checks its order on the new fold before taking it, so no step copies the running fold.
let s = ["a", "b", "c"]
let m = Monoid("", (x: Str, y: Str) => x + y)
print(s.scan(m).all)
