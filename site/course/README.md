# Course authoring

`foundations.json` and `advanced.json` are ordered arrays of lesson metadata.
The corresponding prose lives in `../content/course-*.md`. The build reads
foundations before advanced, validates every exercise reference, and generates
the course catalog and chapter controls. Keep the reference guide intact;
course chapters teach in sequence and link to it for deeper lookups.

## Teach one idea, then test it

Start each chapter with an H2: the template supplies its H1. Derive a concept
from a small contest problem, show a worked example, then place an exercise
before moving on. Explain the likely wrong answer, not just the right syntax.
Keep hints useful without immediately giving away the answer. A solution should
be understandable even after a learner guessed correctly.

A lesson object has `slug`, `title`, `description`, `duration`, and `exercises`.
Its slug is the Markdown filename without `.md`; duration is text such as
`15 min`. Each lesson has at least three exercises. Exercise IDs are globally
unique lowercase words separated by hyphens. Embed an exercise at its teaching
point using an empty fence:

````text
```exercise domains-count-gaps
```
````

## Knowledge checks

An exercise is plain data, not executable source. Prose fields are rendered as
escaped text, not HTML or Markdown. Newlines in hints and solutions are retained.
An optional `code` string is displayed as highlighted CP Lang.

```json
{
  "id": "domains-count-gaps",
  "kind": "choice",
  "title": "Count the boundaries",
  "prompt": "An array has five elements. How many boundary positions does it have?",
  "options": ["Four", "Five", "Six"],
  "answer": 2,
  "explanation": "There is one gap before the first element and one after each element.",
  "hint": "Draw the elements as boxes, then mark every gap, including both ends.",
  "solution": "Six. Five elements have six boundaries. Elements and the gaps around them are different index spaces."
}
```

Answers are zero-based option indices. Use plausible distractors that expose a
specific misconception. Do not imply a knowledge check typechecks a program.

## Output predictions

For a prediction, set `kind` to `output`, `fixture` to an existing example name,
and `tests` to an array of `{input, expected}` text objects. Use at least two
cases when the example has meaningful input. Show the tested source immediately
before the exercise or in its `code` field so the question stands alone.
The available fixtures are `sum`, `bounds`, `accumulation`, `ascii`, `optional`,
and `memo`, corresponding to `../examples/*.cp`.

`test_site.py` compiles the real fixture with the Python prototype and C++17,
runs every case, and compares the result with the answer key. `test_compiler.cjs`
runs all prediction cases through the generated WASM runtime. Do not hardcode an answer for an untested design fragment.
Output checking ignores whitespace only: token order and full integer precision
matter. Browser checks execute the real fixture; each output exercise also has
an editable WASM sandbox. Keep fixtures within the Rust backend's supported
subset, and explicitly label prototype-only behavior such as `@memo` caching.

## Status and verification

Use an `example NAME` fence for native-tested examples. Use a labeled `cp`
fragment for approved design, and say plainly when an API or workflow is planned.
Review against `../../plan/19-index-and-lends.md` and `../../decisions.md`.
In particular, do not invent a working `spec`, `checker`, or `interactor` CLI.

From the repository root:

```sh
python3 site/test_site.py
node --test site/test_course.cjs
node --test site/test_compiler.cjs
python3 site/bundle.py
```

Then check the chapter and offline HTML in Chrome: wrong answer, hint, solution,
correct answer, changed answer, reset, reload, catalog progress, keyboard-only
use, and narrow light/dark layouts. Keep generated HTML out of git.
