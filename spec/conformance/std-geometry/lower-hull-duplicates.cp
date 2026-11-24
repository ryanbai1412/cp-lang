// expect: ok, output "3\n4"
print(lower_hull([Pt(0, 0), Pt(0, 0), Pt(1, -1), Pt(2, 0), Pt(2, 0)]).len)
print(lower_hull([Pt(0, 0), Pt(1, 0), Pt(2, 0), Pt(3, 0)], collinear: true).len)
