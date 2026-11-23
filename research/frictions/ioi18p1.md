# ioi18p1 friction

## An interactive grader task has no local judge for its `extern fn`

- **Where:** the submission is `extern fn press(p: Str) -> Int` plus `@grader fn guess_sequence(n: Int) -> Str`.
  R10.1.3 says local runs use a generated stub, but a stub cannot answer `press`: the answer
  depends on the hidden word, and nothing in the spec lets a file supply the judge half of an
  `extern fn` (an `@interactor` speaks stdin/stdout, not calls). So `cpl run` and `cpl-conform
  corpus` cannot run the real submission; spec.cp plays the sample grader instead (reads S,
  answers presses with a local `fn press`, prints `Accepted: q`).
- **Repro:** `extern fn press(p: Str) -> Int` / `print(press("A"))` checks clean, then `cpl run`
  traps R0001 "this fn has no body: the judge's grader supplies it".
- **Goal it hurt:** doubt to evidence (the real submission can't be run or stressed locally).
- **Routed:** spec worklog (the non-interactive half, no stub calling a `@grader fn`, is already
  in the tools worklog from apio23p1).

## `cpl check` skips what `cpl run` rejects in a `@grader fn` body

- **Where:** the old spec.cp used `Str.of([first])` inside `@grader fn guess_sequence`; `cpl check`
  said "no errors", but `cpl run` on the same body outside `@grader` is E0201 "doesn't know the
  name `Str`". Today's spelling is `Str([first])`.
- **Repro:** `@grader` / `fn f(n: Int) -> Str = Str.of(['a'])` checks clean; without `@grader`
  (plus `print(f(1))`) `cpl check` is E0201.
- **Goal it hurt:** dependable submission (a grader task's body is never checked).
- **Routed:** compiler worklog.
