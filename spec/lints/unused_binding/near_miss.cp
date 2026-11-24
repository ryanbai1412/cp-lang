// A name read only inside a string is still read.
fn next(x: Int) -> Int:
    let y = x + 1
    print("{y}")
    return y
print(next(3))
// A parameter an `@cpp` body reads is read.
fn add(a: Int, b: Int) -> Int = @cpp("return a + b;")
print(add(2, 3))
