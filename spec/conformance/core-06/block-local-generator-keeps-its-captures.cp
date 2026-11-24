// expect: ok, output "2 4 6"
// rule: 06.3.3 07.2.10
// input: "3\n"
// A generator fn declared in a block is one like a top-level fn, and sees the enclosing fn's `n`.
fn evens(n: Int) -> List<Int>:
    fn upto() -> yields Int:
        for i in 1..=n: yield 2 * i
    [x for x in upto()]
let n: 1..=5 = read()
print(evens(n))
