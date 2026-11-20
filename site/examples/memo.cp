let n: 1..=20 = read()

// @memo caching is prototype-only. This runnable version uses plain recursion.
fn f(i: Int) -> Int:
    if i == 0:
        return 1
    f(i - 1) * 3 + i

print(f(n - 1))
