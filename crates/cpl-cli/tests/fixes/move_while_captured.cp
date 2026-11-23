fn demo():
    let xs = [1, 2]
    let f = () => xs.len
    let ys = xs
    print(f() + ys.len)
demo()
