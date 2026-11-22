// Below `@!multitest`, a top-level `return` ends this test case only: the next one starts from its first line.
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0:
            yield i

@!multitest
let n: 0..=20 = read()
if n == 0:
    print("zero")
    return
for i in 0..n:
    for j in 0..n:
        if i * j == 30:
            print("thirty", i, j)
            return              // out of both loops
for x in evens(n):
    if x == 4:
        print("four")
        return                  // out of a generator loop
print(n)
