// expect: ok, output "1\n7\n8\n9\n2"
// rule: 06.3.6
// `yield from e` is `for x in e: yield x`: a list's elements, in order, between other yields.
fn around(xs: List<Int>) -> yields Int:
    yield 1
    yield from xs
    yield 2
for x in around([7, 8, 9]): print(x)
