let s: Str = read()
var h = 0
for c in s:  // expect: hand_rolled_string_algo
    h = (h * 131 + c.ord) % 1000000007
print(h)
