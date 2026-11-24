// expect: prefix_totals_list
// rule: 06.4.10
// A running-total list built by hand gets std's `prefix_totals_list` lint, declared with `@lint` beside `scan` in std, not in the linter (decision 512).
let a = [3, 1, 4]
var p = [0] // <- here
for x in a:
    p.push(p.last! + x)
print(p[3])
