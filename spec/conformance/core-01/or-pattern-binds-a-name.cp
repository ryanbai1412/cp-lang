// expect: ok, output "3\n4\n0"
// rule: 01.7.3
// A name bound in every alternative of a `|` pattern holds the part the matching alternative bound.
fn one(a: ?Int, b: ?Int) -> Int = match (a, b):
    (Some(c), None) | (None, Some(c)) => c
    _ => 0
print(one(Some(3), None))
print(one(None, Some(4)))
print(one(Some(1), Some(2)))
