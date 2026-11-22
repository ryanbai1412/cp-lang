// expect: ok, output "URD"
// rule: 03.2.2

var s = ""
for i in 0..3:
    s += if i == 0: 'U' elif i == 1: 'R' else: 'D'
print(s)
