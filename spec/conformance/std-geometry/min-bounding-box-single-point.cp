// expect: ok, output "YES"
let b = min_bounding_box([Pt(1, 1)])
print(yes(b.area == 0.0 and b.perimeter == 0.0))
