var c: ?Int = None
c max= None
print(c ?? -1)
c max= Some(3)
c max= Some(-5)
c max= None
print(c ?? -1)
var r: [3]?Int = None
r[1] max= Some(7)
r[1] max= Some(2)
r[2] max= None
print(r[0] ?? -1, r[1] ?? -1, r[2] ?? -1)
