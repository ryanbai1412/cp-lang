// expect: E0904
// rule: R03.1.9
// The judge the build is for (here the default, `generic`) reaches the checker as build data:
// an 800 MB array at its bounds is over its 256 MB.
var a: [0..100000000]Int = 0  // <- here
a[5] = 1
print(a[5])
