// expect: ok, output "big small small"
// rule: 01.7.1
// `else` runs when the binder fails or a comma condition is false.
fn f(m: ?Int) -> Str:
    if let v = m, v > 5:
        return "big"
    else:
        return "small"
print(f(Some(9)), f(Some(3)), f(None))
