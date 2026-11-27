// expect: ok, output "1-4"
// rule: 11.6.1
// A program's own `write` is an ordinary fn: `print` still sends its text through std's `write`
// role and reads `io`'s `sep`, while `write(2)` calls the program's fn.
fn write(x: Int) -> Int = x * 2
with io(sep: "-"):
    print(1, write(2))
