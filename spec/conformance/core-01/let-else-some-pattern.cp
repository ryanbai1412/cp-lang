// expect: ok, output "6 0"
// rule: 01.6.2
fn f(x: ?Int) -> Int:
    let Some(y) = x else: return 0
    y + 1
print(f(5), f(None))
