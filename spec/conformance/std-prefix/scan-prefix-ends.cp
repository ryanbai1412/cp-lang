// expect: ok, output "0 8"
let s = [3, 1, 4].scan(+)
print(s.prefix(0), s.prefix(3))
