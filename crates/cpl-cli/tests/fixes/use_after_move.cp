struct Bag(xs: List<Int>)
fn f():
    let a = [1, 2]
    let b = Bag(a)
    let c = a
    print(a.len, b.xs.len, c.len)
f()
