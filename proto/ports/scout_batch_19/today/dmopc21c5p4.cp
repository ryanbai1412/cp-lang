// DMOPC '21 Contest 5 P4, Permutations & Primes (multi-answer constructive, buildable today-form)
// original: corpus/submissions/4223525.cpp
let maxn = 1000005
domain P = 0..=maxn
var is_prime: [P]Bool = true
is_prime[P(0)] = false
is_prime[P(1)] = false
var primes: List<Int> = []

for i in 2..=maxn:
    if is_prime[P(i)]:
        primes.push(i)
        if i * i <= maxn:
            var j = i * i
            while j <= maxn:
                is_prime[P(j)] = false
                j += i

fn find_pair(target: Int) -> (Int, Int):
    for idx in 0..primes.len:
        let a = primes[idx]
        let b = target - a
        if b > a and is_prime[P(b)]:
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
