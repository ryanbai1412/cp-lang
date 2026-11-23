// expect: ok, output "3"
// rule: 01.6.4
var best: ?Int = None
for c in [5, 3, 7]: best min= c
print(best!)
