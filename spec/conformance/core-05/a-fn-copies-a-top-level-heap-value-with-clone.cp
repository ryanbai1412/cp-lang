// expect: ok, output "3 2"
// rule: 05.11.1
// R05.11.1: a fn reads top-level state lent; `var b = base.clone()` is the copy it may change.
let base = [1, 2]
fn grown() -> Int:
    var b = base.clone()
    b.push(3)
    b.len
print(grown(), base.len)
