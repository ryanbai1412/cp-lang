// expect: ok, output "3\n-1"
// rule: 01.6.2
// An `else` block leaves when its last `if`/`match` leaves on every branch.
fn f(x: ?Int, loud: Bool) -> Int:
    let v = x else:
        if loud:
            return -1
        else:
            return -2
    v
fn g(x: ?Int) -> Int:
    let v = x else:
        match x:
            Some(_) => todo
            None => return -1
    v
print(f(Some(3), true))
print(g(None))
