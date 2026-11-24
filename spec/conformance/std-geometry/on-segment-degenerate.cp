// expect: ok, output "YES NO NO"
print(yes(on_segment(Pt(2, 2), Pt(2, 2), Pt(2, 2))), yes(on_segment(Pt(2, 2), Pt(2, 2), Pt(2, 2), strict: true)), yes(on_segment(Pt(1, 2), Pt(0, 0), Pt(2, 2))))
