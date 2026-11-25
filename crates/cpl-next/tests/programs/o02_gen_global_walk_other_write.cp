// expect: ok, output "3 6"
// Legal: body writes a different global from the one walked.
var xs: List<Int> = [1, 2, 3]
var ys: List<Int> = []
for x in xs: ys.push(x * 2)
print(ys.len, ys[2])
