// here: 4
// C++ reads into an indexed place: "no `cin` in scope", with no fix.
var a = List.filled(3, 0)
for (int i = 0; i < 3; i++) cin >> a[i];
print(a)
