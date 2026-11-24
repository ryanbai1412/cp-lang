// expect: ok, output "5 5"
// rule: 05.10.1
// A nested fn's closure that reads a variable the nested fn captured by reference sees it from inside the fn it is passed to, as from a call in place; interpreter and C++ agree.
domain T = [3]
fn outer():
    var price: [T]?Int = None
    price[T.first!] = 5
    fn pay(xs: List<T>) -> Int = xs.sum(of: x => price[x]!)
    fn pay_by(xs: List<T>) -> Int:
        let f = (x: T) => price[x]!
        return xs.sum(of: f)
    print(pay([T.first!]), pay_by([T.first!]))
outer()
