// A slice whose end is before its start stops the program: status 3, not an empty list.
let a = [1, 2, 3, 4]
let (lo, hi): (Int, Int) = read()
print("slicing", a[1..3])
print(a[lo..hi])
print("never printed")
