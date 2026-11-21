let xs = [3, 1, 4, 1, 5]
fn big(x: Int) -> Bool: x > 2
print(count(xs, x => x > 2), count(xs, big))
print(any(xs, x => x > 4), all(xs, x => x > 1), all(xs, x => x > 0))
print(first(xs, x => x > 3), first(xs, x => x > 9))
print(prod(xs), prod(x + 1 for x in xs), prod([2.0, 0.5]))
let ys = List<Int>()
print(prod(ys), count(ys, big), first(0..10, i => i * i > 20))
print(count("hello", c => c == 'l'))
print(xs.count(x => x > 2), xs.count(1), xs.any(x => x > 4), xs.all(x => x > 0))
print("banana".count(c => c == 'a'), (0..10).count(i => i % 3 == 0))
