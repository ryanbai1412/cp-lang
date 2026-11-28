// expect: ok, output "-1 0 1 -4"
// engines: next
print(Int(Sign.Neg), Int(Sign.Zero), Int(Sign.Pos), Int((-4).sign) * 4)
