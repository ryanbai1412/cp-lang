// expect: ok, output "1 2 3 1 2"
// rule: 05.3.5 05.7.4
// A map-walk entry is lent, so `var w = v` copies it; pushing to `w` leaves the map unchanged.
var m: Map<Int, List<Int>> = Map()
m[1] = [1, 2]
for (k, v) in m:
    var w = v
    w.push(3)
    print(w, m[k])
