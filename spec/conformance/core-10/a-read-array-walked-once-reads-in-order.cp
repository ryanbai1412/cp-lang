// expect: ok, output "30\n69\n71\n82"
// rule: 10.2.3
// input: "1 2 3 4 5\n6 7 8\n9 1 2"
// A generator over a read array, handed to a fn that walks it once, reads each value in that
// fn's loop; a loop that reads more inside reads the whole array first, so input stays in order.
domain V = 0..5
domain W = 0..3
fn take<S: Seq<Int>>(xs: S) -> Int:
    var s = 0
    for x in xs: s += x
    s
print(take(x * 2 for x in read<[V]0..=9>()))
for x in read<[W]0..=9>():
    let y = read<0..=9>()
    print(x * 10 + y)
