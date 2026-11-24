// expect: ok, output "5 0\n10\n6\n12"
// rule: 05.10.1
// A nested fn holds an outer `var` by reference; a closure it makes over that `var` and passes to another fn reads it there.
let n = 3
domain T = [n]
fn apply(f: Int -> Int, x: Int) -> Int = f(x)
fn go():
    var price: [T]?Int = None
    price[T.first!] = 5
    fn pay(m: Int):
        let types: List<T> = [T.first!]
        let (known, unknown) = types.partition(x => price[x] != None)
        print(m - known.sum(of: x => price[x]!), unknown.len)
    pay(10)
    var cost = [5, 1, 0]
    let total = (m: Int) =>
        let p = cost[0]
        m - apply(i => cost[i], 0) + p
    print(total(10))
    fn bump():
        cost[1] += apply(i => cost[i], 0)
        print(cost[1])
    bump()
    fn twice():
        cost[2] = [0, 1].sum(of: i => cost[i])
        print(cost[2] + cost[1] - cost[0])
    twice()
go()
