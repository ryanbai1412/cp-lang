let prices = [7, 2, 4]
var cur: ?Int = Some(20)
var spent = 0
while (cur ?? 0) > 2:
    let c = cur!
    spent += c
    cur = if c > 3: Some(c / 2) else: None
print(spent)
let best = prices.find(x => x > 5)
if (best ?? 0) > 4:
    let b = best!
    print(b * 2)
