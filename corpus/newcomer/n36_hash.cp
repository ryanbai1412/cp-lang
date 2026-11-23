let s: Str = read()
var h = 0
for c in s:
    h = (h * 131 + c.ord) % 1000000007  // expect: hand_rolled_string_algo
print(h)
