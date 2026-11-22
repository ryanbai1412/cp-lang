// expect: ok, output "0 0 5"
// rule: 02.6.2
struct Tally:
    var top = 0
    var sum = 0
    cap: Int = 5
let t = Tally()
print(t.top, t.sum, t.cap)
