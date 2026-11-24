var a = [1]
var b: ?Int = nullptr  // <- here
print(b ?? a[0])
