// expect: ok, output "YES"
print(yes(Dir8.all().map(d => d.delta) == [(-1, 0), (-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1)] and Dir8.from("abcdefgh", 'c') == Some(Dir8.all()[2]) and Dir8.from("abcdefgh", 'x') == None and Dir8.all()[0].cw.delta == (-1, 1) and Dir8.all()[7].cw == Dir8.all()[0] and Dir8.all()[0].ccw == Dir8.all()[7] and Dir8.all()[1].back.delta == (1, -1)))
