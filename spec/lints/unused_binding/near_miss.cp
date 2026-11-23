// A name read only inside a string is still read.
fn next(x: Int) -> Int:
    let y = x + 1
    print("{y}")
    return y
print(next(3))
