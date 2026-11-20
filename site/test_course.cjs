const {test} = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const vm = require("node:vm");
const {check} = require("./assets/course.js");

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
