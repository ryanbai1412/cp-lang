// `x /= b` and `x %= b` follow the same Euclidean rule as `/` and `%`, also for a negative divisor,
// and a `where` on a word read is checked.
let s: [domain S]Char = read(alphabet: "ab") where S.len <= 5
let x: -100..=100 = read()
let b: -9..=-2 = read()
var q = x
q /= b
var r = x
r %= b
print(q, r, x / b, x % b, S.len)
var digits: List<Int> = []
var left = x
while left != 0:
    digits.push(left % b)
    left /= b
print(digits)
