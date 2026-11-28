// expect: ok, output "5 5 0"
fn f(d: Int) -> Int:
    if d > 0: d + 1 else: 0
fn h(d: ?Int) -> Int:
    if let k = d: k + 1 else: 0
print(f(4), h(Some(4)), h(None))
