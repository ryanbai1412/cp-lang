// A. Problem from Dolgoderevenskoye
// https://codeforces.com/contest/2289/problem/A
// odd divisors O, even divisors a*O for x = 2^a * odd; Artem wins iff O != a*O, i.e. a != 1
@!multitest
let x: 1..=1e18 = read()
print(yes(x % 2 == 1 or x % 4 == 0))
