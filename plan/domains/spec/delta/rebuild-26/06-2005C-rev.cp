// 2005C Lazy Narek, revised: the target word is read per test (length 2..=10)
// "narek" is matched. A matched letter is +1, any other narek letter -1; an
// unfinished tail of g letters is worth -2g at the end.

@!multitest
let (n, m): (1..=1e3, 1..=1e3) = read()
let word = read<Str>()
let pieces = read<[n]Str>()
domain Got = 0..word.len
var best: [Got]?Int = None
best[Got(0)] = 0
for s in pieces:
    var next = best.clone()
    for g in Got:
        if best[g] == None: continue
        var have = Int(g)
        var score = best[g]!
        for ch in s:
            if ch == word[have]:
                have = (have + 1) % word.len
                score += 1
            elif word.any(w => w == ch):
                score -= 1
        next[Got(have)] max= score
    best = next
var answer = 0
for g in Got:
    if best[g] != None: answer max= best[g]! - 2 * Int(g)
print(answer)
