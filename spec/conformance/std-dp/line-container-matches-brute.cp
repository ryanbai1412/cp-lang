// expect: ok, output "4 3 YES 2\n0 1 YES\nYES"
use std.dp.{Line, LineContainer}
var c = LineContainer(max)
c.add(Line(1, 0))
c.add(Line(-1, 4))
print(c.at(0) ?? -1, c.at(3) ?? -1, yes(c.line_at(3) == Some(Line(1, 0))), c.len)
var d = LineContainer(min)
d.add(Line(1, 0))
d.add(Line(-1, 4))
print(d.at(0) ?? -1, d.at(3) ?? -1, yes(LineContainer(min).at(5) == None))
var ok = true
var seed = 5
for t in 0..40:
    let up = t % 2 == 0
    var h = if up: LineContainer(max) else: LineContainer(min)
    var lines: List<Line> = []
    for _ in 0..1 + t % 17:
        seed = (seed * 1103515245 + 12345) % 2147483648
        let l = Line(seed % 11 - 5, seed / 11 % 41 - 20)
        h.add(l)
        lines.push(l)
        for x in -12..13:
            var want = lines[0].k * x + lines[0].m
            for q in lines:
                let v = q.k * x + q.m
                if (up and v > want) or (not up and v < want): want = v
            if h.at(x) != Some(want): ok = false
            let got = h.line_at(x)!
            if got.k * x + got.m != want: ok = false
    if h.len < 1 or h.len > lines.len: ok = false
print(yes(ok))
