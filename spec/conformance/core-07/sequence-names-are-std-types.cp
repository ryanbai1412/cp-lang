// expect: ok, output "12 6 6"
// rule: 07.2.18
// `Seq` is a std trait a signature can name: a generator, a reversed view and a list all pass it, being std types that implement it (decision 509).
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0: yield i
fn total(s: Seq<Int>) -> Int = s.sum()
print(total(evens(7)), total([1, 2, 3].rev), total([1, 2, 3]))
