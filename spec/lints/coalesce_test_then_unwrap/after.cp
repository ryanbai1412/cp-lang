let prices = [7, 2, 4]
var cur: ?Int = Some(20)
var spent = 0
while let c = cur, c > 2:
    spent += c
    cur = if c > 3: Some(c / 2) else: None
print(spent)
let best = prices.find(x => x > 5)
if let b = best, b > 4:
    print(b * 2)
