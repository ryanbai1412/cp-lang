// expect: 3 1 200 7
// rule: separate calls to a wrapper returning compress() get distinct fresh domains
fn wrap(a: List<Int>):
    compress(a)

let (Dwa, wa) = wrap([100, 200, 300])
let (Dwb, wb) = wrap([7])
print(Dwa.len, Dwb.len, wa[wa.at(200)], wb[wb.at(7)])
