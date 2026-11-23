// here: 4
// Python's `"{:.2f}".format(x)` (self-run mistakes): a string names its values in its holes.
let x = 3.14159
print("{} = {:.2f}".format("x", x))
