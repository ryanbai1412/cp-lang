// A local fn's `mut` params of each layout (Int, ?Int, a list) go back to
// the caller's places: a frame local, a top-level var and a list element.
var total = 0
var best: ?Int = None
var rows = [[1], [2]]
fn main():
    var count = 0
    for round in 0..3:
        fn bump(mut n: Int, mut m: ?Int, mut xs: List<Int>, by: Int):
            n += by + round
            m = Some(n)
            xs.push(n)
        bump(mut count, mut best, mut rows[round % 2], 1)
        bump(mut total, mut best, mut rows[0], 10)
        print(count, total, best ?? -1, rows)
main()
