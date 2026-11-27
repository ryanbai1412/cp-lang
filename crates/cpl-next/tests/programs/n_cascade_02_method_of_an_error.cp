// expect: E0201
// rule: diagnostics
// lacks: "has no"
// lacks: "expected"
// A method called on an unknown name is not looked up, and its result is not a type to infer.
print(nope.windows(2).last!) // <- here
