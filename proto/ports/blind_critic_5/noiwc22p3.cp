// Runnable CP Lang v2.2 port of noiwc22p3 (NOIWC '22 P3 - Wordle)
// Original C++: corpus/submissions/4261177.cpp
// Runs directly under the Rust `cpl run` and `cpl build` (c521b2c).
// Implements the exact stateful init/guess protocol over stdin/stdout so the
// differential harness drives both 4261177.cpp and noiwc22p3.cp through the
// exact same sequence of Wordle games and compares deterministic filtering & scoring.
// Workarounds vs blind first draft (noiwc22p3_draft.cp):
// - No `@grader fn` declarations, `for ... if`, or `min_by` in Rust prelude -> explicit command loop & scan
// - Keeps deterministic order (no random shuffle) to compare exact optimal minimax scores

var dict: List<Str> = []
var cand: List<Str> = []
var last_guess: Str = "leant"

fn reset(initial_letter: Char):
    cand.clear()
    for w in dict:
        if w[0] == initial_letter:
            cand.push(w)

fn eval_code(guess: Str, ans: Str) -> Int:
    var gold: [5]Bool = false
    var silver: [5]Bool = false
    for i in 0..5:
        gold[i] = guess[i] == ans[i]
    for i in 0..5:
        if not gold[i]:
            for j in 0..5:
                if not gold[j] and guess[i] == ans[j]:
                    silver[i] = true
    var code = 0
    for i in 0..5:
        code = code * 2 + Int(gold[i])
    for i in 0..5:
        code = code * 2 + Int(silver[i])
    code

fn make_code(gold_str: Str, silver_str: Str) -> Int:
    var code = 0
    for i in 0..5:
        code = code * 2 + Int(gold_str[i] == '1')
    for i in 0..5:
        code = code * 2 + Int(silver_str[i] == '1')
    code

fn guess_step(rem: Int, init_ch: Char, gold_s: Str, silver_s: Str) -> Str:
    if rem == 5:
        reset(init_ch)
        last_guess = "leant"
        return last_guess
    if rem == 0:
        return "paper"

    let target = make_code(gold_s, silver_s)
    var filtered: List<Str> = []
    for w in cand:
        if eval_code(last_guess, w) == target:
            filtered.push(w)
    cand = filtered
    assert cand.len > 0

    if cand.len == 1:
        last_guess = cand[0]
        return last_guess

    var best_score: ?Int = none
    var best_word: Str = "aaaaa"
    for a in dict:
        var cnts: [1024]Int = 0
        var max_bucket = 0
        for b in cand:
            let h = eval_code(a, b)
            cnts[h] += 1
            if cnts[h] > max_bucket:
                max_bucket = cnts[h]
        var empty_groups = 0
        for h in 0..1024:
            if cnts[h] == 0:
                empty_groups += 1
        let score = max_bucket * 10000 + empty_groups
        if best_score == none or score < (best_score ?? 0) or (score == (best_score ?? 0) and a < best_word):
            best_score = score
            best_word = a
    last_guess = best_word
    last_guess

let num_words: Int = read()
for _ in 0..num_words:
    let w: Str = read()
    dict.push(w)

let num_queries: Int = read()
for _ in 0..num_queries:
    let rem: Int = read()
    let init_s: Str = read()
    let gold_s: Str = read()
    let silver_s: Str = read()
    print(guess_step(rem, init_s[0], gold_s, silver_s))
