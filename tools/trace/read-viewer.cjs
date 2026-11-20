// Test helpers extract the authored source that the CLI actually embeds.
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const html = fs.readFileSync(path.join(__dirname, "index.html"), "utf8");
function section(id, tag = "script") {
  const match = html.match(new RegExp(`<${tag} id="${id}">([\\s\\S]*?)</${tag}>`));
  if (!match) throw new Error(`Missing canonical viewer section: ${id}`);
  return match[1];
}
// Use this realm so strict equality works for returned arrays and records.
const core = new vm.Script(`(() => { ${section("trace-core")}\nreturn TraceCore; })()`).runInThisContext();
module.exports = {html, section, core};
