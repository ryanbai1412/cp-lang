// expect: ok, output "10 16 2"
// rule: R06.1.7
// A nested fn hoists within its block: it may be called above its statement once its captures are set.
fn g(n: Int) -> Int:
    let k = n * 2
    var hits = 0
    let r = twice(k)
    fn twice(x: Int) -> Int = add(add(x))
    fn add(x: Int) -> Int:
        hits += 1
        x + k / 4
    r + hits - 4
let xs = [1, 2]
print(g(4), g(6), xs.len)
