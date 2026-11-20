/* Knowledge checks, output predictions, and WASM-graded source exercises. */
const CourseChecks = (() => {
  const normalize = value => String(value).trim().split(/\s+/).filter(Boolean).join(" ");
  function check(exercise, answer) {
    if (exercise.kind === "choice") {
      if (!/^(0|[1-9]\d*)$/.test(String(answer)))
        return {passed:false, message:"Choose an answer first."};
      const passed = Number(answer) === exercise.answer;
      return {passed, message:passed ? "Correct." + (exercise.explanation ? " " + exercise.explanation : "")
        : "Not quite. Revisit the example or open a hint, then try again."};
    }
    if (exercise.kind === "code") {
      // A completion receipt contains the exact source and its observed stdout.
      if (!answer || typeof answer.source !== "string" || !answer.source.trim())
        return {passed:false, message:"Write a program, then check it."};
      return check({...exercise, kind:"output"}, answer.outputs);
    }
    if (exercise.kind === "output") {
      if (!Array.isArray(answer) || answer.length !== exercise.tests.length)
        return {passed:false, message:"Fill in the output for each case."};
      const matches = exercise.tests.map((test, i) => normalize(answer[i]) === normalize(test.expected));
      const passed = matches.every(Boolean);
      return {passed, matches, message:passed
        ? `All ${matches.length} cases match.`
        : `${matches.filter(Boolean).length} of ${matches.length} cases match. Check the marked cases or open a hint. Whitespace does not matter.`};
    }
    return {passed:false, message:"This check is unavailable."};
  }
  async function execute(exercise, answer, run) {
    if (exercise.kind === "code") {
      if (typeof answer !== "string" || !answer.trim())
        return {passed:false, message:"Write a program, then check it."};
      const outputs = [];
      for (const [i, test] of exercise.tests.entries()) {
        const result = await run(answer, test.input);
        if (!result.ok) return {passed:false, message:`Case ${i + 1}: ${result.stderr || "The program could not run."}`};
        outputs.push(result.stdout);
      }
      const receipt = {source:answer, outputs};
      const result = check(exercise, receipt);
      if (result.passed) result.message += " Executed locally with CP Lang WebAssembly.";
      else {
        const i = result.matches.indexOf(false);
        result.message = `Case ${i + 1} did not match. Expected: ${exercise.tests[i].expected.trim() || "(empty output)"}. Got: ${outputs[i].trim() || "(empty output)"}.`;
      }
      return {...result, answer:receipt};
    }
    if (exercise.kind !== "output") return check(exercise, answer);
    const tests = [];
    for (const test of exercise.tests) {
      const result = await run(exercise.source, test.input);
      if (!result.ok) return {passed:false, message:result.stderr || "The program could not run."};
      tests.push({expected:result.stdout});
    }
    const result = check({...exercise, tests}, answer);
    if (result.passed) result.message += " Executed locally with CP Lang WebAssembly.";
    return result;
  }
  return {normalize, check, execute};
})();

if (typeof document !== "undefined" && document.getElementById("course-data")) {
  (() => {
    "use strict";
    const data = JSON.parse(document.getElementById("course-data").textContent);
    const prefix = "cp-course-v1:";
    const records = new Map();
    let storageAvailable = true;
    // Read only known IDs and bound record sizes. Corrupt/obsolete data is ignored.
    for (const [id, revision] of Object.entries(data.revisions)) {
      let raw;
      try {
        raw = localStorage.getItem(prefix + id);
      } catch (_) { storageAvailable = false; continue; }
      try {
        if (raw && raw.length < 20000) {
          const record = JSON.parse(raw);
          if (record && record.revision === revision && record.passed === true &&
              CourseChecks.check(data.exercises[id], record.answer).passed)
            records.set(id, record);
        }
      } catch (_) { /* A damaged record must not prevent other checks loading. */ }
    }
    function save(id, answer, passed) {
      if (passed) records.set(id, {revision:data.revisions[id], answer, passed:true});
      else records.delete(id);
      try {
        if (passed) localStorage.setItem(prefix + id, JSON.stringify(records.get(id)));
        else localStorage.removeItem(prefix + id);
      } catch (_) { storageAvailable = false; }
      updateProgress();
    }
    function count(ids) { return ids.filter(id => records.has(id)).length; }
    function updateProgress() {
      document.querySelectorAll("[data-progress]").forEach(region => {
        const lessons = data.lessons.filter(lesson => region.dataset.progress === "all" || lesson.slug === region.dataset.progress);
        const ids = lessons.flatMap(lesson => lesson.exercises);
        const done = count(ids);
        region.querySelector("progress").value = done;
        region.querySelector("[data-progress-label]").textContent = `${done} of ${ids.length} exercises checked`;
        region.querySelector(".storage-note").textContent = storageAvailable
          ? "Progress stays in this browser. No account needed."
          : "Browser storage is unavailable. Progress lasts for this page only.";
      });
      document.querySelectorAll("[data-lesson]").forEach(label => {
        const lesson = data.lessons.find(item => item.slug === label.dataset.lesson);
        const done = count(lesson.exercises);
        label.textContent = done === lesson.exercises.length ? "Complete" : `${done} / ${lesson.exercises.length}`;
        label.dataset.complete = String(done === lesson.exercises.length);
      });
    }
    document.querySelectorAll("[data-exercise]").forEach(section => {
      const id = section.dataset.exercise, exercise = data.exercises[id];
      const form = section.querySelector("form"), status = section.querySelector(".exercise-feedback");
      const badge = section.querySelector("[data-badge]");
      let revision = 0;
      function readAnswer() {
        if (exercise.kind === "code") return form.querySelector("[data-source]").value;
        return exercise.kind === "choice" ? form.querySelector("input:checked")?.value
          : [...form.querySelectorAll("textarea")].map(field => field.value);
      }
      function feedback(result) {
        status.textContent = result.message;
        section.dataset.state = result.passed ? "passed" : "retry";
        badge.textContent = result.passed ? "Checked" : "Try again";
        form.querySelectorAll(exercise.kind === "code" ? "[data-source]" : "textarea").forEach((field, i) => {
          if (exercise.kind === "code") field.setAttribute("aria-invalid", String(!result.passed));
          else if (result.matches) field.setAttribute("aria-invalid", String(!result.matches[i]));
          else field.removeAttribute("aria-invalid");
        });
      }
      const record = records.get(id);
      if (record) {
        if (exercise.kind === "choice") {
          const field = [...form.querySelectorAll("input")].find(input => input.value === String(record.answer));
          if (field) field.checked = true;
        } else if (exercise.kind === "code") {
          form.querySelector("[data-source]").value = record.answer.source;
        } else if (Array.isArray(record.answer)) {
          form.querySelectorAll("textarea").forEach((field, i) => { field.value = String(record.answer[i] ?? ""); });
        }
        const result = CourseChecks.check(exercise, record.answer);
        if (result.passed) feedback(result);
        else records.delete(id);
      }
      form.addEventListener("submit", async event => {
        event.preventDefault();
        const current = ++revision, answer = readAnswer();
        const submit = form.querySelector('[type="submit"]');
        submit.disabled = true;
        section.dataset.state = "checking";
        status.textContent = exercise.kind !== "choice" ? "Running each case with CP Lang WebAssembly…" : "Checking…";
        try {
          const runnable = exercise.kind === "output" && !exercise.source
            ? {...exercise, source: section.querySelector("[data-source]")?.defaultValue ?? ""}
            : exercise;
          const result = await CourseChecks.execute(runnable, answer,
            (source, input) => CpCompiler.call("run", source, input));
          if (revision !== current) return;
          feedback(result);
          save(id, result.answer ?? answer, result.passed);
        } catch (error) {
          if (revision === current) { feedback({passed:false, message:error.message}); save(id, null, false); }
        } finally { submit.disabled = false; }
      });
      form.addEventListener("input", event => {
        if (exercise.kind === "code" && !event.target.matches("[data-source]")) return;
        revision++;
        if (!section.dataset.state && !records.has(id)) return;
        delete section.dataset.state;
        status.textContent = "Answer changed. Check it again when ready.";
        badge.textContent = "Not checked";
        form.querySelectorAll("textarea").forEach(field => field.removeAttribute("aria-invalid"));
        save(id, null, false);
      });
      form.addEventListener("reset", () => {
        revision++;
        delete section.dataset.state;
        status.textContent = "Answer and progress cleared.";
        badge.textContent = "Not checked";
        form.querySelectorAll("textarea").forEach(field => field.removeAttribute("aria-invalid"));
        section.querySelectorAll("details").forEach(details => { details.open = false; });
        save(id, null, false);
      });
    });
    updateProgress();
  })();
}

if (typeof module !== "undefined") module.exports = CourseChecks;
