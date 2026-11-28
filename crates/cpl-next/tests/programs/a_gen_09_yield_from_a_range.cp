// expect: ok, output "0\n1\n2\n10\n12"
// rule: R06.3.6
// `yield from` over a range and over a chain: any Seq, not only a list or a generator value.
fn spread(n: Int) -> yields Int:
    yield from 0..n
    let evens = (10..14).filter(x => x % 2 == 0)
    yield from evens
for x in spread(3): print(x)
