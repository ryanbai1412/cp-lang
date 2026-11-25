// expect: ok, output "13"
// Two closures over one var: inc and get must see the same variable (spec R05.10.1 example).
fn go() -> Int:
    var cnt = 0
    let inc = () => cnt += 1
    let get = () => cnt
    inc()
    let a = get()
    inc()
    inc()
    a * 10 + get()
print(go())
