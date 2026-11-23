let n: 1..=100 = read()
let a: [0..n]Int = read()
var found = -1  // expect: missing sentinel-instead-of-optional
for i in 0..n:  // expect: search_loop
    if a[i] == 0:
        found = i
        break
print(found)
