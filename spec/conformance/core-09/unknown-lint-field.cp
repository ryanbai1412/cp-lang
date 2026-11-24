// expect: L0001
// rule: 09.1.5
with lint(float_equals: allow): // <- here
    print(yes(0.5 + 0.25 == 0.75))
