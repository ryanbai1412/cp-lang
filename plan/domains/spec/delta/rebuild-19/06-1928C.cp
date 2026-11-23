// CF 1928C Physical Education Lesson: the period 2k - 2 divides n - x (rising part) or
// n + x - 2 (falling part), with k >= x.
@!multitest
let (n, x) = read<(2..=1e9, 1..=1e9)>()
let fits = (d: Int) => d % 2 == 0 and d / 2 + 1 >= x
let rising = divisors(n - x).count(fits)
let falling = divisors(n + x - 2).count(d => fits(d) and (n - x) % d != 0)
print(rising + falling)
