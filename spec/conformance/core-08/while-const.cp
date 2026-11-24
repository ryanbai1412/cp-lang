// expect: E0107
// rule: 08.2.9
var i = 0
while const true: // <- here
    i += 1
    if i > 3: break
print(i)
