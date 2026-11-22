// expect: ok, output "YES"
let u = Dir4.all()[0]
print(yes(u.cw.delta == (0, 1) and u.ccw.delta == (0, -1) and u.back.delta == (1, 0) and Dir4.all()[3].cw == u and Dir4.from("URDL", 'D') == Some(Dir4.all()[2])))
