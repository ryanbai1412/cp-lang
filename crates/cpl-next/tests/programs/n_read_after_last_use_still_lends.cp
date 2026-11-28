// expect: E0603
// rule: R05.5.3
// `out` is read after `Some(out)`, so that call is not its last use and `r` lends it.
fn g(n: Int) -> ?List<Int>:
    var out = [n]
    let r = Some(out)
    print(out.len)
    return r // <- here
print(match g(2):
    Some(xs) => xs.len
    None => 0)
