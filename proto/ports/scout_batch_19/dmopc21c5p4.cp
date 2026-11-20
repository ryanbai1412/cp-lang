// DMOPC '21 Contest 5 P4, Permutations & Primes — ideal CP Lang sketch
// Category: Multi-answer Constructive + Custom Checker (multi-test sum-budget gen + -1 sentinel)
// Original C++: corpus/submissions/4223525.cpp (299 LOC raw incl. 130-line custom IO, ~130 LOC stripped)
// Buildable today-form: proto/ports/scout_batch_19/today/dmopc21c5p4.cp (verified 106 cases up to 10000)

let maxn = 1_000_005
let sieve = Sieve(maxn) // stdlib prime sieve: sieve.is_prime(x), sieve.primes

fn solve_one(n: 1..=1e6) -> ?List<1..=n>:
    if n == 1: return [1]
    if n <= 3: return none
    if n == 4: return [3, 1, 4, 2]
    let target = if n == 5 or n == 7 or (n % 2 == 0 and n != 6): n else: n - 1
    let a = sieve.primes.find(p => target - p > p and sieve.is_prime(target - p))!
    let b = target - a
    var ans: List<1..=n> = []
    var cur = 0
    for _ in 0..n:
        ans.push(cur + 1)
        cur = if cur + a < n: cur + a else: cur - b
    ans

let t: 1..=1e5 = read()
// WISH B19-3: multi-test sum-budget constraint `sum(ns) <= 1e6` expressed directly on the input vector
// so `cpl stress` and `cpl bench` generate valid worst-case partitions (e.g. many small + one huge n)
let ns: [domain Case = 0..t](1..=1e6) = read()
assume sum(ns) <= 1_000_000

for c in Case:
    print(solve_one(ns[c]), none: -1)

// WISH B19-7: per-case checker on multi-test output, validating permutation + prime differences + -1
checker:
    for c in Case:
        let n = ns[c]
        let out: ?[0..n](1..=n) = read_output(none: -1)
        let solvable = n != 2 and n != 3
        match out:
            none =>
                require not solvable, f"returned -1 for solvable n={n}"
            some(p) =>
                require solvable, f"returned permutation for impossible n={n}"
                require is_permutation(p, 1..=n)
                require all(sieve.is_prime(abs(p[i + 1] - p[i])) for i in 0..(n - 1))
