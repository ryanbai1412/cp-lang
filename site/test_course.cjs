const {test} = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const vm = require("node:vm");
const {check, execute} = require("./assets/course.js");

test("catalog progress rechecks stored answers even when passed and revision are current", () => {
  const data = {
    lessons:[{slug:"lesson", exercises:["choice", "output"]}],
    exercises:{choice:{kind:"choice", answer:1, explanation:"Correct."},
      output:{kind:"output", tests:[{expected:"3"}, {expected:"4"}]}},
    revisions:{choice:"current", output:"current"}
  };
  function catalog(answers) {
    const progress = {value:0}, label = {}, note = {};
    const region = {dataset:{progress:"all"}, querySelector:selector =>
      ({progress, "[data-progress-label]":label, ".storage-note":note})[selector]};
    const context = {
      document:{getElementById:() => ({textContent:JSON.stringify(data)}),
        querySelectorAll:selector => selector === "[data-progress]" ? [region] : []},
      localStorage:{getItem:key => JSON.stringify({revision:"current", passed:true,
        answer:answers[key.slice("cp-course-v1:".length)]})}
    };
    vm.runInNewContext(fs.readFileSync(require.resolve("./assets/course.js"), "utf8"), context);
    return progress.value;
  }
  assert.equal(catalog({choice:"0", output:["3", "wrong"]}), 0);
  assert.equal(catalog({choice:"1", output:["3", "wrong"]}), 1);
  assert.equal(catalog({choice:"1", output:[" 3\n", "4"]}), 2);
});

test("a choice needs an explicit answer; empty input never selects option zero", () => {
  const question = {kind:"choice", answer:0, explanation:"Boundaries are gaps."};
  for (const answer of [undefined, null, "", " ", false, "0.0", "-0", "0oops"])
    assert.equal(check(question, answer).passed, false, String(answer));
  assert.equal(check(question, "0").passed, true);
  assert.equal(check(question, "1").passed, false);
  assert.match(check(question, "0").message, /Boundaries are gaps/);
});

test("output checks compare every case, preserve integers, and ignore only whitespace", () => {
  const question = {kind:"output", tests:[
    {expected:"9007199254740993\n-1\n"}, {expected:"0 10 10"}, {expected:""}
  ]};
  assert.deepEqual(check(question, [" 9007199254740993 -1 ", "0\n10\t10", "  "]).matches, [true,true,true]);
  assert.equal(check(question, ["9007199254740992 -1", "0 10 10", ""]).passed, false);
  assert.deepEqual(check(question, ["9007199254740993 -1", "0 10", "extra"]).matches, [true,false,false]);
  assert.equal(check(question, ["9007199254740993 -1"]).passed, false);
  assert.equal(check(question, "9007199254740993 -1").passed, false);
});

test("a blank prediction is not zero and token order matters", () => {
  const question = {kind:"output", tests:[{expected:"0"}, {expected:"1 2"}]};
  assert.equal(check(question, ["", "1 2"]).passed, false);
  assert.equal(check(question, ["0", "2 1"]).passed, false);
  assert.equal(check(question, ["0", "1 2"]).passed, true);
});

test("unknown check kinds fail closed", () => {
  assert.equal(check({kind:"source"}, "print(42)").passed, false);
});

test("output predictions execute the source and compare real stdout, not answer fixtures", async () => {
  const exercise = {kind:"output", source:"print(read() * 2)", tests:[{input:"21", expected:"wrong fixture"}]};
  const calls = [];
  const result = await execute(exercise, ["42"], async (source, input) => {
    calls.push([source, input]); return {ok:true, stdout:"42\n"};
  });
  assert.equal(result.passed, true);
  assert.deepEqual(calls, [[exercise.source, "21"]]);
  assert.equal((await execute(exercise, ["42"], async () => ({ok:false, stderr:"error[E0301]"}))).passed, false);
});

test("editing a first prediction clears pending feedback even if its run times out", async () => {
  const handlers = {}, submit = {disabled:false}, status = {}, badge = {};
  const field = {value:"14", setAttribute(){}, removeAttribute(){}};
  const form = {
    querySelector:() => submit,
    querySelectorAll:() => [field],
    addEventListener(name, callback) { handlers[name] = callback; }
  };
  const section = {dataset:{exercise:"output"},
    querySelector:selector => ({form, ".exercise-feedback":status, "[data-badge]":badge})[selector]};
  const data = {lessons:[], revisions:{output:"current"}, exercises:{output:{kind:"output",
    source:"print(14)", tests:[{input:"", expected:"14"}]}}};
  let rejectRun;
  const context = {
    document:{getElementById:() => ({textContent:JSON.stringify(data)}),
      querySelectorAll:selector => selector === "[data-exercise]" ? [section] : []},
    localStorage:{getItem:() => null, removeItem(){}, setItem(){}},
    CpCompiler:{call:() => new Promise((resolve, reject) => { rejectRun = reject; })}
  };
  vm.runInNewContext(fs.readFileSync(require.resolve("./assets/course.js"), "utf8"), context);
  const pending = handlers.submit({preventDefault(){}});
  assert.equal(submit.disabled, true);
  assert.match(status.textContent, /Running each case/);
  field.value = "changed";
  handlers.input();
  assert.match(status.textContent, /Answer changed/);
  rejectRun(new Error("Stopped after 5 seconds."));
  await pending;
  assert.equal(submit.disabled, false);
  assert.match(status.textContent, /Answer changed/);
  field.value = "14";
  context.CpCompiler.call = async () => ({ok:true, stdout:"14\n"});
  await handlers.submit({preventDefault(){}});
  assert.equal(section.dataset.state, "passed");
  assert.equal(badge.textContent, "Checked");
});

test("output prediction check falls back to sandbox defaultValue even after the user edits the sandbox", async () => {
  const handlers = {}, submit = {disabled:false}, status = {}, badge = {};
  const field = {value:"42", setAttribute(){}, removeAttribute(){}};
  const sandboxSource = {defaultValue:"print(42)\n", value:"print(999)\n"};
  const form = {
    querySelector:() => submit,
    querySelectorAll:() => [field],
    addEventListener(name, callback) { handlers[name] = callback; }
  };
  const section = {dataset:{exercise:"output"},
    querySelector:selector => ({
      form,
      ".exercise-feedback":status,
      "[data-badge]":badge,
      "[data-source]":sandboxSource,
    })[selector]};
  const data = {lessons:[], revisions:{output:"current"}, exercises:{output:{kind:"output",
    tests:[{input:"", expected:"42"}]}}};
  const calls = [];
  const context = {
    document:{getElementById:() => ({textContent:JSON.stringify(data)}),
      querySelectorAll:selector => selector === "[data-exercise]" ? [section] : []},
    localStorage:{getItem:() => null, removeItem(){}, setItem(){}},
    CpCompiler:{call:async (op, source, input) => {
      calls.push([op, source, input]);
      return {ok:true, stdout:"42\n"};
    }}
  };
  vm.runInNewContext(fs.readFileSync(require.resolve("./assets/course.js"), "utf8"), context);
  await handlers.submit({preventDefault(){}});
  assert.equal(section.dataset.state, "passed");
  assert.deepEqual(calls, [["run", "print(42)\n", ""]]);
});
