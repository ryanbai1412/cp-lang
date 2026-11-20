let n: 1..=5 = read()
print(n)
@checker
fn check():
    let x: Int = read_output()
    assert x == n
