// expect: ok, output "42"
// `let f` sees later writes to a captured heap var (reference capture).
fn go() -> Int:
    var xs: List<Int> = [1]
    let first = () => xs[0]
    xs = [42, 43]
    first()
print(go())
