// error: `const` picks an `if` branch at compile time
var i = 0
while const true:
    i += 1
    if i > 3: break
print(i)
