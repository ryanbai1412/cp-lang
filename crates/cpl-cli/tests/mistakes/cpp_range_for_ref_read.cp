// here: 4
// C++ reads into each item through a reference: help only, then `x = read()` on a copy.
var a = List.filled(3, 0)
for (auto &x : a) cin >> x;
print(a)
