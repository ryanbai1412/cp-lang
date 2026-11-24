// expect: ok, output "1\n2"
// Every `Window` journey from std-api/R13 is one short call.
var w = Window(min)
// journey: push
for x in [3, 1, 2]: w.push(x)
// journey: fold of the window
print(w.all!)
// journey: pop the oldest
w.pop(3)
w.pop(1)
print(w.all!)
