// expect: E0601
// rule: 05.10.3
fn go():
    var a = [1, 2]
    fn bump(mut x: Int):
        a[0] += 1
        x += 1
    bump(mut a[0]) // <- here
go()
