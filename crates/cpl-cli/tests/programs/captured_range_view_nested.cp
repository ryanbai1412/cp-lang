// A bag range view captured two closures deep keeps its bounds, open ones included.
fn main():
    var b = SortedBag.of([1, 3, 5])
    let v = b.range(2..)
    let g = (k: Int) =>
        let h = () => [x * k for x in v].len
        h() + v.len
    print(g(2), g(3))
main()
