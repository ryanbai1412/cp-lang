// expect: ok, output "6 14 6"
// rule: R06.2.1
// A top-level lambda's body binds a lambda with parameters of its own; each level types its own.
let f = (x: Int) =>
    let g = (y: Int) => y * 2
    g(x)
let h = (a: Int) =>
    let k = (b: Int) =>
        let m = (c: Int) => c + b
        m(a)
    k(a + 3)
let n = ((v: Int) => ((w: Int) => w * v)(v + 1))(2)
print(f(3), h(5) + 1, n)
