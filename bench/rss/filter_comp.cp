let n: 1..=8_000_000 = read()
let a = read<[n]0..=1_000_000_000>()
let even = [(x, 2 * x) for x in a if x % 2 == 0]
let few = [(x, 2 * x) for x in a if x % 1000 == 0]
print(even.len, few.len, even[even.len - 1].1 + few[0].1)
