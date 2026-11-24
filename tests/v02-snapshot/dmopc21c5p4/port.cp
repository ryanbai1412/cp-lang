// Source: DMOPC '21 Contest 5 P4 (Permutations & Primes), https://dmoj.ca/problem/dmopc21c5p4
// Problem: given N, print a permutation of 1..N whose adjacent gaps |p[i+1] - p[i]| are all
//   prime and whose signed differences p[i+1] - p[i] take as few distinct values as possible,
//   or -1 if none exists. Up to T tests, T, N <= 1e6, sum of N <= 1e6.
// Solution: one difference can never work, so aim for two: +a and -b with primes a < b.
//   Take a + b = N when N is even (not 6) or N is 5 or 7, else a + b = N - 1 (Goldbach;
//   scan the sieve's primes for a). Walk from 1, stepping +a while it stays <= N, else -b;
//   as gcd(a, b) = 1 this visits every value once. N = 1 prints 1, N = 2, 3 print -1, and
//   N = 4 is 3 1 4 2. One sieve to 1e6, then O(N) per test plus the prime scan.
// original: corpus/submissions/4223525.cpp

let maxn = 1000005
let sv = Sieve.new(maxn)

fn find_pair(target: Int) -> (Int, Int):
    for a in sv.primes:
        let b = target - a
        if b > a and sv.is_prime(b):
            return (a, b)
    (0, 0)

fn solve_one(n: Int):
    if n == 1:
        print(1)
        return
    if n <= 3:
        print(-1)
        return
    if n == 4:
        print(3, 1, 4, 2)
        return
    let target = if n == 5 or n == 7 or (n % 2 == 0 and n != 6): n else: n - 1
    let ab = find_pair(target)
    let a = ab.0
    let b = ab.1
    var ans: List<Int> = []
    var cur = 0
    for i in 0..n:
        ans.push(cur + 1)
        if cur + a < n:
            cur += a
        else:
            cur -= b
    print(ans)

let t: 1..=1e5 = read()
for _ in 0..t:
    let n: 1..=1e6 = read()
    solve_one(n)
