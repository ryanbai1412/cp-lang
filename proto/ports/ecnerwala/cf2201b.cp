// Codeforces 2201 B: construct a length-2N array containing each value 1..N twice
// with exactly K distinct adjacent pair sums when N <= K <= 2N - 1.
@!multitest
let (n_in, k): (Int, Int) = read()
if k < n_in or k > 2 * n_in - 1:
    print("NO")
else:
    print("YES")
    let n2 = k - n_in + 1
    var n = n_in
    var out: List<Int> = []
    while n > n2:
        n -= 1
        out.push(n + 1)
        out.push(n + 1)
    if n == 1:
        out.push(1)
        out.push(1)
    else:
        out.push(1)
        out.push(2)
        for i in 1..n - 1:
            out.push(i + 2)
            out.push(i)
        out.push(n - 1)
        out.push(n)
    print(out)
