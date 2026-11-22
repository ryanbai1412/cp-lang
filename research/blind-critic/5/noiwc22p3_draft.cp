// Problem: noiwc22p3 (NOI Winter Camp 2022 P3 - Wordle)
// Original: corpus/submissions/4261177.cpp
// Blind Critic #5 — First Draft (strictly from docs/one-page-cheat-sheet.md v2.2)

domain Slot = 0..5
type Mask = Subset<Slot>
type Feedback = (gold: Mask, silver: Mask)

var dict: List<Str> = []
var cand: List<Str> = []
var last_guess = "leant"

fn evaluate(guess: Str, ans: Str) -> Feedback:
    var gold = Mask.empty
    var silver = Mask.empty
    for i in Slot:
        if guess[i.to_idx] == ans[i.to_idx]:
            gold = gold.with(i)
    for i in Slot if i not in gold:
        for j in Slot if j not in gold:
            if guess[i.to_idx] == ans[j.to_idx]:
                silver = silver.with(i)
    return (gold, silver)

fn to_mask(bits: [Slot]Bool) -> Mask:
    var m = Mask.empty
    for i in Slot if bits[i]:
        m = m.with(i)
    return m

@grader fn init(num_scramble: Int, scramble: Str):
    dict.clear()
    for i in 0..num_scramble:
        dict.push(scramble[(i * 5)..(i * 5 + 5)])

@grader fn guess(
    num_testcase: Int,
    remaining_guesses: Int,
    initial_letter: Char,
    gold: [Slot]Bool,
    silver: [Slot]Bool,
) -> Str:
    shuffle(mut dict)
    if remaining_guesses == 5:
        cand = [w.copy() for w in dict if w[0] == initial_letter]
        last_guess = "leant"
        return last_guess.copy()
    elif remaining_guesses == 0:
        return "paper"
    else:
        let target: Feedback = (to_mask(gold), to_mask(silver))
        cand = [w.copy() for w in cand if evaluate(last_guess, w) == target]
        assert not cand.is_empty

    if cand.len == 1:
        last_guess = cand[0].copy()
        return last_guess.copy()

    let best = min_by(dict, a => score(a))!
    last_guess = best.copy()
    return last_guess.copy()

fn score(a: Str) -> (Int, Str):
    var cnts = HashMap<Feedback, Int>()
    for b in cand:
        cnts[evaluate(a, b)] += 1
    let worst = max(v for (_, v) in cnts.items) ?? 0
    let empty_groups = 1024 - cnts.len
    return (worst * 10_000 + empty_groups, a.copy())
