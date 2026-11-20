"""Course content contract and accessible, server-rendered exercise controls."""

import html
import json
from pathlib import Path
import re
from compiler import editor_html

ROOT = Path(__file__).resolve().parent
MANIFESTS = ("foundations", "advanced")
FIXTURES = {
    "sum",
    "bounds",
    "accumulation",
    "ascii",
    "optional",
    "memo",
    "early_return",
    "multitest_divider",
    "enum_match",
    "collections_heap",
    "sort_bsearch",
    "lends",
    "tropical_dp",
    "optional_coalesce",
    "memo_dp",
    "segtree_monoid",
    "lazy_action",
    "fenwick_sum",
    "dsu_mod",
    "colocated_brute",
}


def load_lessons():
    lessons, ids, slugs = [], set(), set()
    for manifest in MANIFESTS:
        lessons.extend(json.loads((ROOT / "course" / f"{manifest}.json").read_text()))
    for lesson in lessons:
        name = lesson["slug"]
        if not re.fullmatch(r"course-[a-z-]+", name) or name in slugs:
            raise ValueError(f"Invalid or duplicate lesson: {name}")
        slugs.add(name)
        for key in ("title", "description", "duration"):
            if not isinstance(lesson[key], str) or not lesson[key].strip():
                raise ValueError(f"Missing {key} in {name}")
        if len(lesson["exercises"]) < 3:
            raise ValueError(f"Lesson {name} needs at least three exercises")
        source = (ROOT / "content" / f"{name}.md").read_text()
        referenced = re.findall(r"```exercise ([a-z0-9-]+)\s*\n```", source)
        expected = [e["id"] for e in lesson["exercises"]]
        if sorted(referenced) != sorted(expected):
            raise ValueError(f"Exercise directives differ from manifest: {name}")
        for exercise in lesson["exercises"]:
            key = exercise["id"]
            if not re.fullmatch(r"[a-z][a-z0-9-]+", key) or key in ids:
                raise ValueError(f"Invalid or duplicate exercise: {key}")
            ids.add(key)
            for field in ("title", "prompt", "hint", "solution"):
                if not isinstance(exercise[field], str) or not exercise[field].strip():
                    raise ValueError(f"Missing {field} in {key}")
            if exercise["kind"] == "choice":
                options, answer = exercise["options"], exercise["answer"]
                if (len(options) < 2 or len(set(options)) != len(options)
                        or type(answer) is not int or not 0 <= answer < len(options)):
                    raise ValueError(f"Invalid choices in {key}")
                if not exercise.get("explanation"):
                    raise ValueError(f"Missing explanation in {key}")
            elif exercise["kind"] in ("output", "code"):
                if not exercise["tests"]:
                    raise ValueError(f"Missing test cases in {key}")
                if exercise["kind"] == "output" and exercise["fixture"] not in FIXTURES:
                    raise ValueError(f"Invalid output fixture in {key}")
                if exercise["kind"] == "code":
                    for field in ("starter", "reference"):
                        if not isinstance(exercise[field], str) or not exercise[field].strip():
                            raise ValueError(f"Missing {field} in {key}")
                    if len(exercise["tests"]) < 3:
                        raise ValueError(f"Code exercise {key} needs at least three cases")
                for case in exercise["tests"]:
                    if not all(isinstance(case[field], str) for field in ("input", "expected")):
                        raise ValueError(f"Output cases must be text: {key}")
            else:
                raise ValueError(f"Unknown exercise kind: {key}")
    return lessons


def exercise_html(exercise, highlight):
    e = html.escape
    key, kind = exercise["id"], exercise["kind"]
    code, sandbox = "", ""
    if exercise.get("code"):
        code = f'<pre class="exercise-code" tabindex="0"><code>{highlight(exercise["code"], "cp")}</code></pre>'
    if kind == "choice":
        fields = '<fieldset><legend>Choose one answer</legend>' + "".join(
            f'<label class="exercise-option"><input type="radio" name="{key}-answer" value="{i}"><span>{e(option)}</span></label>'
            for i, option in enumerate(exercise["options"])
        ) + '</fieldset>'
        label = "Knowledge check"
    elif kind == "output":
        fields = "".join(
            f'<div class="exercise-case"><h4>Case {i + 1}</h4><span class="case-label">Standard input</span>'
            f'<pre tabindex="0">{e(case["input"]) or "(no input)"}</pre>'
            f'<label for="{key}-{i}">Your predicted output</label>'
            f'<textarea id="{key}-{i}" data-case="{i}" rows="2" spellcheck="false" autocapitalize="off" autocomplete="off" aria-describedby="{key}-prompt"></textarea></div>'
            for i, case in enumerate(exercise["tests"])
        )
        label = "Predict the output"
        source = (ROOT / "examples" / f'{exercise["fixture"]}.cp').read_text()
        sandbox = (
            '<details class="exercise-help exercise-sandbox"><summary>Edit and run this program</summary>'
            '<p class="exercise-sandbox-note">This sandbox is independent. Check answer uses the original program and the inputs shown above.</p>'
            + editor_html(key + "-sandbox", source, exercise["tests"][0]["input"])
            + '</details>'
        )
    else:
        label = "Write code"
        fields = (
            '<p class="exercise-sandbox-note">Run program uses the editable input. '
            'Check code runs your source against every test case below.</p>'
            + editor_html(key, exercise["starter"], exercise["tests"][0]["input"])
            + '<details class="exercise-help"><summary>Grading test cases</summary>'
            + ''.join(
                f'<div class="exercise-case"><h4>Case {i + 1}</h4>'
                f'<span class="case-label">Standard input</span><pre>{e(case["input"]) or "(no input)"}</pre>'
                f'<span class="case-label">Expected output</span><pre>{e(case["expected"])}</pre></div>'
                for i, case in enumerate(exercise["tests"])
            ) + '</details>'
        )
    solution = f'<p>{e(exercise["solution"])}</p>'
    if kind == "code":
        solution += f'<pre class="exercise-code" tabindex="0"><code>{highlight(exercise["reference"], "cp")}</code></pre>'
    submit_label = "Check code" if kind == "code" else "Check answer"
    return (
        f'<section class="exercise" id="{key}" data-exercise="{key}" aria-labelledby="{key}-title">'
        f'<div class="exercise-heading"><span>{label}</span><span class="exercise-badge" data-badge>Not checked</span></div>'
        f'<h3 id="{key}-title">{e(exercise["title"])}</h3>'
        f'<p id="{key}-prompt">{e(exercise["prompt"])}</p>{code}'
        f'<form class="exercise-form">{fields}<div class="exercise-actions">'
        f'<button class="primary-button" type="submit">{submit_label}</button>'
        '<button class="secondary-button" type="reset">Reset</button></div></form>'
        f'<div class="exercise-feedback" id="{key}-feedback" role="status" aria-live="polite"></div>'
        f'{sandbox}'
        f'<details class="exercise-help"><summary>Hint</summary><p>{e(exercise["hint"])}</p></details>'
        f'<details class="exercise-help"><summary>Solution and explanation</summary>{solution}</details>'
        '<noscript>Enable JavaScript to check your answer. Hints and solutions work without it.</noscript></section>\n'
    )


def catalog_html(lessons):
    cards = []
    for i, lesson in enumerate(lessons, 1):
        name = lesson["slug"]
        cards.append(
            f'<li><a class="lesson-card" href="{name}.html"><span class="lesson-number">{i:02}</span><span>'
            f'<strong>{html.escape(lesson["title"])}</strong><span class="lesson-description">{html.escape(lesson["description"])}</span>'
            f'<span class="lesson-meta">{html.escape(lesson["duration"])} · {len(lesson["exercises"])} exercises</span></span>'
            f'<span class="lesson-completion" data-lesson="{name}">0 / {len(lesson["exercises"])}</span></a></li>'
        )
    return '<ol class="course-lessons">' + "".join(cards) + '</ol>'


def progress_html(lessons, current=None):
    selected = [lesson for lesson in lessons if lesson["slug"] == current] if current else lessons
    total = sum(len(lesson["exercises"]) for lesson in selected)
    scope = current or "all"
    label = "Chapter progress" if current else "Course progress"
    return (f'<div class="course-progress" data-progress="{scope}"><div><strong>{label}</strong>'
            f'<span data-progress-label>0 of {total} exercises checked</span></div>'
            f'<progress max="{total}" value="0" aria-label="{label}"></progress>'
            '<small class="storage-note">Progress stays in this browser. No account needed.</small></div>')
