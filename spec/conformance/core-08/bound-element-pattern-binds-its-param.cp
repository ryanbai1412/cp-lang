// expect: ok, output "3 3"
// rule: 08.3
// `S: Seq<?T>` binds `T` to what is inside the element's `?`, whatever the param is named.
fn somes<S: Seq<?T>, T>(xs: S) -> List<T>:
    var out: List<T> = []
    for x in xs:
        if let y = x: out.push(y)
    out
fn somes_u<S: Seq<?U>, U>(xs: S) -> List<U>:
    var out: List<U> = []
    for x in xs:
        if let y = x: out.push(y)
    out
print(somes([Some(1), None, Some(2)]).sum(), somes_u([Some(1), Some(2)]).sum())
