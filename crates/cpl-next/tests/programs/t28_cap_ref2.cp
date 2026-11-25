// expect: ok, output "6"
// input: "5\n"
var total = 0
for i in 0..3:
    var cnt = 0
    let inc = () => cnt += 1
    inc()
    inc()
    total += cnt
print(total)
