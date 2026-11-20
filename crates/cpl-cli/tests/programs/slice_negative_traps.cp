// A slice bound below 0 stops the program: `a[..-1]` is not "all but the last".
let a = [1, 2, 3, 4]
let (lo, hi): (Int, Int) = read()
print("slicing", a[1..3])
print(a[lo..hi])
print("never printed")
