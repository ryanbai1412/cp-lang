// expect: R0401 at 3
// input: "10000"
// rule: 09.3.7
// A failed `assert` in a fn on the fast interpreter stops the run with its message after more than 64 KiB already printed.
fn check(k: Int) -> Int:
    assert k < 9000, "k is {k}"
    return k
let n: Int = read()
for k in 0..n:
    print("line", check(k))
