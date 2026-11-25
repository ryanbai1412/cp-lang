// expect: E0603
// (spec) R8 block rule, as k06: QUESTION
var fs: List<() -> Int> = []
for i in 0..3:
    var c = i * 10
    fs.push(() => c)  // <- here
    c += 1
var s = 0
for i in 0..3: s = s * 100 + fs[i]()
print(s)
