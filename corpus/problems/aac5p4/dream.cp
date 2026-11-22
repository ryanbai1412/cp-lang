// AAC 5 P4: the most moves for n and x is n / 2, plus one for odd n when x's parity matches
// the residue rule; print the count and then 1..=count.
let (n, x) = read (Int, Int)
let extra = n % 2 == 1 and n > 1 and (x % 2 == 0) == ((n - 3) / 2 % 4 in [0, 3])
let k = if n == 1: 0 else: n / 2 + extra.int
print(k)
print(1..=k, sep: "\n")
