// SAC 22 CC3 J2: is the segment along the x-axis, the y-axis or neither?
let (a, b) = read ((x: Int, y: Int), (x: Int, y: Int))
print(if a.y == b.y: "x-axis" elif a.x == b.x: "y-axis" else: "neither")
