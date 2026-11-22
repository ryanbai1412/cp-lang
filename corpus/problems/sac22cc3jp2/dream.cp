// sac22cc3jp2: is the segment between two points parallel to the x-axis, the y-axis, or neither?
let (x1, y1, x2, y2) = read<(Int, Int, Int, Int)>()
print(if y1 == y2: "x-axis" elif x1 == x2: "y-axis" else: "neither")
