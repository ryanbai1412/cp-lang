fn f(xs: List<Int>):
    let a = xs.fin(0) ?? return print(0)
    let b = xs.sorte().map(x => x + 1)
    print(a, b)
