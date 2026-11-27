// expect: ok, output "5 9\n6"
// rule: R05.10.1
// A lambda inside a nested fn binds its own locals; their types are the enclosing fn's, not the nested fn's table.
fn go():
    var price: List<?Int> = [5, None, 4]
    fn pay():
        let f = (x: Int) =>
            let p = price[x]!
            p
        let g = (xs: List<Int>) => xs.sum(of: x => price[x]!)
        print(f(0), g([0, 2]))
    pay()
    fn count() -> Int:
        let (known, _) = [0, 1, 2].partition(x => price[x] != None)
        known.len
    print(count() * 3)
go()
