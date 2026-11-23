let top = 1_000_000
let q: 1..=2e5 = read()
let s = Sieve(top)
let prime: [0..=top]Int = [0..=top] x => if s.is_prime(x): 1 else: 0
let pre = prime.scan(+)
for _ in 0..q:
    let x: 1..=top = read()
    print(pre.fold(0..=x))
