// expect: ok, output "6"
// (spec) R4: a value argument the call also writes is copied in (conformance core-05/param-unchanged-by-a-callee-writing-its-source).
// The generator is handed a closure that writes the list the generator walks (through a shared var).
fn each(xs: List<Int>, f: () -> ()) -> yields Int:
    for x in xs:
        f()
        yield x
fn go() -> Int:
    var xs: List<Int> = [1, 2, 3]
    let pop = () => xs = []
    var s = 0
    for x in each(xs, pop): s += x
    s
print(go())
