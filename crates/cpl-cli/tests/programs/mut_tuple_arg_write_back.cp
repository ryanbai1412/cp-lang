// A tuple local passed as a `mut` arg comes back changed, whether the fn
// returns nothing or a value.
fn rotate(mut q: (Int, Int, Int), by: Int):
    q = (q.1 + by, q.2, q.0)
fn swap(mut q: (Int, Int, Int)) -> Int:
    q = (q.2, q.1, q.0)
    q.0 + q.1
fn main():
    var p = (1, 2, 3)
    for k in 0..4:
        rotate(mut p, k)
        let s = swap(mut p)
        print(p.0, p.1, p.2, s)
main()
