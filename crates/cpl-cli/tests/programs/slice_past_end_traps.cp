// A slice that ends past the list stops the program.
let a = [1, 2, 3, 4]
let (lo, hi): (Int, Int) = read()
print("slicing", a[1..3])
print(a[lo..hi])
print("never printed")
