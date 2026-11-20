// `print` evaluates every argument, left to right, before it writes anything: a question asked by a later
// argument is on its own line, before the answer.
var log: List<Int> = []
fn q(x: Int) -> Int:
    print("?", x)
    x * 2
fn note(x: Int) -> Int:
    log.push(x)
    x
print("!", q(3), note(1), [q(4), 5])
print("a", "b", 1 + 2)
print(log)
