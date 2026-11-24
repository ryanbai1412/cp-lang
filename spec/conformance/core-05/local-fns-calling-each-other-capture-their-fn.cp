// expect: ok, output "17 13 YES NO"
// rule: 05.10.6
fn walk(n: Int, k: Int) -> (Int, Int):
    fn down(i: Int) -> Int = if i <= 0: 0 else: k + up(i - 1)
    fn up(i: Int) -> Int = if i <= 0: 0 else: 1 + down(i - 1)
    (down(n), up(n))
fn parity(n: Int) -> Bool:
    fn even(i: Int) -> Bool = if i == 0: true else: odd(i - 1)
    fn odd(i: Int) -> Bool = if i == 0: false else: even(i - 1)
    even(n)
let (a, b) = walk(5, 5)
print("{a} {b} {yes(parity(10))} {yes(parity(7))}")
