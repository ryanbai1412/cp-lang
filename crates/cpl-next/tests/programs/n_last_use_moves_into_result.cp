// expect: ok, output "2\n3\n7"
// rule: R05.5.3
// An owned local passed whole where it is read for the last time moves into the result, so
// returning it lends nothing: `r` holds `out`, then `r` is returned. A write to a cell of
// `out` before that is an earlier read, not its last one.
fn f(n: Int) -> ?List<Int>:
    var out = [n]
    out.push(1)
    let r = Some(out)
    return r
fn g(n: Int) -> ?List<Int>:
    if let got = f(n): return Some(got)
    None
fn h(n: Int) -> ?List<Int>:
    var out = [0, 0]
    out[0] = n
    Some(out)
print(match f(2):
    Some(xs) => xs.len
    None => 0)
print(match g(2):
    Some(xs) => xs.len + 1
    None => 0)
print(match h(7):
    Some(xs) => xs[0]
    None => 0)
