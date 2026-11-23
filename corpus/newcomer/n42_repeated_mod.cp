let n: 1..=100000 = read()
var f = 1
for i in 1..=n:
    f = f * i % 1000000007  // expect: manual_mod_reduce
var g = 1
for i in 1..=n:
    g = g * 2 % 1000000007  // expect: repeated_literal
print(f, g)
