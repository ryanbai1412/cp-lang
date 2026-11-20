// A failed `assert` stops the program with status 3, after what was
// printed before it.
let n: Int = read()
print("checking", n)
assert n > 0
print("positive")
assert n % 2 == 0, "{n} is odd"
print("never printed")
