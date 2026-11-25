// expect: E0602
// rule: R05.10.3
// R05.10.3: mut captures are exclusive for the call: passing `mut a` to a fn value that also writes captured `a`.
fn go() -> Int:
    var a = [1, 2]
    let add = (mut xs: List<Int>) => { a.push(9); xs.push(1) }
    add(mut a)  // <- here
    a.len
print(go())
