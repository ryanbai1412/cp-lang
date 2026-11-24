// expect: E0603
// rule: 05.3.5
fn make(n: Int) -> List<Int>:
    var xs = List<Int>()
    for i in 0..n: xs.push(i)
    xs                                  // moves out: O(1)

fn same(xs: List<Int>) -> List<Int> = xs // <- here
// error: returns the lent parameter xs; return xs.clone(), or declare `lends List<Int>`
print(make(3).len)
