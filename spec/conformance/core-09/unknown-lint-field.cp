// expect: L0001 at 1
// rule: 09.1.5
with lint(float_equals: allow):
    print(yes(0.5 + 0.25 == 0.75))
