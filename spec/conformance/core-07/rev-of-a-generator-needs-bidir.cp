// expect: E0606
// rule: 07.2.18
// `.rev` is std's `fn rev<S: Bidir>`: a generator's `Gen<T>` is not `Bidir`, so the error is the plain bound error naming the trait (decision 509).
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0: yield i
print(evens(7).rev.sum()) // <- here
