// expect: ok, output "6 -1\n0\n2\n2\n4\n4\n14"
// rule: R01.6.2
// `let v = x else:` binds the payload after the statement; the block leaves by `return`, `continue` or `break`.
fn f(x: ?Int) -> Int:
    let v: Int = x else: return -1
    v + 1
fn first_even(xs: List<Int>) -> ?Int:
    for x in xs:
        if x % 2 == 0: return x
    None
print(f(5), f(None))
var seen = 0
for k in 0..6:
    let e = first_even([k, k + 1]) else: continue
    let half = if e > 4: None else: Some(e)
    let h = half else: break
    print(h)
    seen += 1
let last = first_even([3, 9]) else:
    print(seen + 9)
    return
print(last)
