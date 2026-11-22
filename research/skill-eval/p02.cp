let n = read<1..=2e5>()
let movies = read<[n](a: Int, b: Int)>()
var end_time = 0
var ans = 0
for m in movies.sorted(by: m => m.b):
    if m.a >= end_time:
        ans += 1
        end_time = m.b
print(ans)
