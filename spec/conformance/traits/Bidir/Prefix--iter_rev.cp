// expect: ok, output "6\n4\n3"
let pre = [3, 1, 2].scan(+)
for t in pre.iter_rev(): print(t)
