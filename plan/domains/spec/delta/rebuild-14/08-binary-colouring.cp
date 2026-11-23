// Codeforces 1977B Binary Colouring
// in: t; per case x. out: per case n, then a_0..a_{n-1} in {-1, 0, 1} with no two adjacent nonzero
@!multitest
var x = read<Int>()
var digits: List<Int> = []
repeat 31:
    if x % 2 == 0:
        digits.push(0)
    elif x % 4 == 1:
        digits.push(1)
        x -= 1
    else:
        digits.push(-1)
        x += 1
    x /= 2
print(digits.len)
print(digits)
