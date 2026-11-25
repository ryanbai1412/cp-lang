// expect: E0603
// rule: R05.10.2, R8
// A closure over a `mut` parameter outlives the call through a global.
var keep = () => 0
fn arm(mut xs: List<Int>):
    keep = () => { xs.push(1); xs.len }  // <- here
fn go():
    var local = [1, 2, 3]
    arm(mut local)
go()
print(keep())
