// Run the real inline core with Node's built-in runner: no test-only copy.
const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const html = fs.readFileSync(path.join(__dirname, 'index.html'), 'utf8');
const core = html.match(/<script id="trace-core">([\s\S]*?)<\/script>/)[1];
// The UI is syntactically valid even though these tests deliberately use no DOM.
new vm.Script(html.match(/<script id="trace-ui">([\s\S]*?)<\/script>/)[1]);
const T = vm.runInNewContext(`${core}\nTraceCore`);
const sample = name => JSON.parse(fs.readFileSync(path.join(__dirname, 'samples', `${name}.json`), 'utf8'));
const plain = value => JSON.parse(JSON.stringify(value));

for (const name of ['knapsack', 'dijkstra', 'tree', 'grid', 'array']) {
  test(`${name} is a valid full-snapshot trace`, () => {
    const trace = sample(name);
    assert.equal(T.validate(trace), trace);
    assert.ok(trace.runs.fast.steps.length > 10);
    for (const s of trace.runs.fast.steps) assert.deepEqual(Object.keys(s.values).sort(), Object.keys(trace.variables).sort());
  });
}

test('finds the precise first DP divergence despite twice as many spec events', () => {
  const trace = sample('knapsack');
  const result = T.analyze(trace);
  assert.equal(result.pairs.length, 34);
  assert.equal(trace.runs.fast.steps.length, 34);
  assert.equal(trace.runs.slow.steps.length, 66);
  const first = result.divergences[0];
  assert.equal(first.fastIndex, 4);
  assert.equal(first.slowIndex, 8);
  assert.equal(first.checkpoint, 'cell:1:4');
  assert.deepEqual(plain(first.differences), [{path:[1,4],fast:6,slow:3,fastPresent:true,slowPresent:true,variable:'dp'}]);
  assert.ok(result.pairs.slice(0, 4).every(p => p.differences.length === 0));
  const last = result.divergences.at(-1);
  assert.ok(last.differences.some(d => d.variable === 'answer' && d.fast === 12 && d.slow === 10));
});

test('does not align internal steps or claim incomparable runs agree', () => {
  const trace = sample('knapsack');
  delete trace.runs.fast.steps[0].checkpoint;
  const result = T.analyze(T.validate(trace));
  assert.equal(result.byFast.has(0), false);
  for (const s of trace.runs.fast.steps) delete s.checkpoint;
  assert.equal(T.analyze(T.validate(trace)).pairs.length, 0);
});

test('agrees when the capture is identical, not just when outputs match', () => {
  const trace = sample('knapsack');
  trace.runs.slow = structuredClone(trace.runs.fast);
  assert.equal(T.analyze(T.validate(trace)).divergences.length, 0);
});

test('diffs object contents, not insertion order; array positions matter', () => {
  assert.equal(T.diff({a:1,b:{x:2}},{b:{x:2},a:1}).length, 0);
  assert.deepEqual(plain(T.diff([1,2],[2,1])).map(d=>d.path), [[0],[1]]);
  assert.deepEqual(plain(T.diff([[0,2]],[[0,3]])).map(d=>d.path), [[0,1]]);
});

test('preserves false, zero, null and out-of-scope as different states', () => {
  assert.equal(T.diff(false,0).length,1);
  assert.equal(T.diff(null,0).length,1);
  assert.equal(T.diff(null,null).length,0);
  const result = T.diff({x:null},{});
  assert.equal(result[0].fastPresent,true);
  assert.equal(result[0].slowPresent,false);
  assert.deepEqual(plain(T.diff([1],[1,2]))[0],{path:[1],slow:2,fastPresent:false,slowPresent:true});
});

test('ignores different implementation locals and honors explicit compare list', () => {
  const trace = sample('knapsack');
  trace.runs.slow = structuredClone(trace.runs.fast);
  trace.runs.slow.steps[0].locals = {implementation:'slow'};
  trace.runs.slow.steps[0].values.weights = [99];
  assert.equal(T.analyze(T.validate(trace)).divergences.length,0);
  delete trace.compare;
  assert.equal(T.analyze(T.validate(trace)).divergences[0].fastIndex,0);
});

test('source names and path keys are not evaluated', () => {
  assert.equal(T.pathName('dp',[1,4]),'dp[1][4]');
  assert.equal(T.pathName('object',['a-b','x',0]),'object["a-b"].x[0]');
  assert.equal(T.pathName('dist',[1],{labels:[1,2,3]}),'dist[2]');
  assert.equal(T.pathName('g',[0,2],{rowLabels:[1,2],colLabels:[5,6,7]}),'g[1][7]');
  assert.equal(T.pathName('best',[4],{labels:[1,2]}),'best[4]');
  assert.equal(T.pathName('dp',[1,4],{rowLabels:['∅','w=2'],colLabels:[0,1,2,3,4]}),'dp[1][4]');
});

test('selecting a parent cell retains exact nested spec differences',()=>{
  const differences=T.diff([{cost:6,count:2}],[{cost:3,count:1}]).map(d=>({...d,variable:'arr'}));
  const selected=T.selectDifference(differences,'arr',[0]);
  assert.deepEqual(plain(selected.path),[0,'cost']);
  assert.equal(selected.fast,6);
  assert.equal(selected.slow,3);
  assert.equal(T.selectDifference(differences,'arr',[0,'count']).fast,2);
  assert.equal(T.selectDifference(differences,'arr',[1]),undefined);
  assert.equal(T.selectDifference(differences,'other',[0]),undefined);
  assert.equal(T.containsPath([1],[10]),false);
});

test('sample algorithms end with independently known results', () => {
  assert.deepEqual(sample('dijkstra').runs.fast.steps.at(-1).values.dist,[0,3,2,7,5,10,9]);
  assert.deepEqual(sample('tree').runs.fast.steps.at(-1).values.size,[7,3,3,1,1,1,1]);
  assert.equal(sample('grid').runs.fast.steps.at(-1).values.distance[5][7],12);
  assert.deepEqual(sample('array').runs.fast.steps.at(-1).values.a,[1,2,3,4,5,6,7,8]);
});

const invalid = [
  ['schema version',t=>t.schemaVersion=9,/schemaVersion/],
  ['missing run',t=>delete t.runs.fast,/Runs/],
  ['empty run',t=>t.runs.fast.steps=[],/at least one/],
  ['bad kind',t=>t.variables.dp.kind='html',/descriptor/],
  ['bad labels',t=>t.variables.dp.rowLabels='rows',/labels/i],
  ['duplicate event id',t=>t.runs.fast.steps[1].id=0,/unique/],
  ['source zero',t=>t.runs.fast.steps[0].line=0,/line/],
  ['source past end',t=>t.runs.fast.steps[0].line=1000,/line/],
  ['duplicate checkpoint',t=>t.runs.fast.steps[1].checkpoint='init',/checkpoint/],
  ['unordered checkpoints',t=>{const a=t.runs.slow.steps[2];t.runs.slow.steps[2]=t.runs.slow.steps[4];t.runs.slow.steps[4]=a;},/same order/],
  ['unknown compared variable',t=>t.compare=['missing'],/compare/],
  ['unknown snapshot variable',t=>t.runs.fast.steps[0].values.missing=1,/unknown/],
  ['ragged matrix',t=>t.runs.fast.steps[0].values.dp=[[0],[0,1]],/rectangular/],
  ['missing focus cell',t=>t.runs.fast.steps[1].focus.path=[99,99],/outside/],
  ['bad focus path',t=>t.runs.fast.steps[1].focus.path='[0]',/focus/],
  ['bad array',t=>t.runs.fast.steps[0].values.weights={},/array/],
  ['bad scalar',t=>t.runs.fast.steps[0].values.answer={},/scalar/],
];
for(const [name,change,expected] of invalid)test(`rejects ${name} before loading`,()=>{const trace=sample('knapsack');change(trace);assert.throws(()=>T.validate(trace),expected);});

test('validates graph endpoints and unique ids',()=>{
  let trace=sample('dijkstra');trace.runs.fast.steps[0].values.graph.edges[0].to='missing';assert.throws(()=>T.validate(trace),/endpoint/);
  trace=sample('dijkstra');trace.runs.fast.steps[0].values.graph.nodes[1].id='0';assert.throws(()=>T.validate(trace),/duplicate/);
  trace=sample('dijkstra');trace.runs.fast.steps[0].values.graph.nodes[0].x='onclick';assert.throws(()=>T.validate(trace),/coordinate/);
});

test('rejects cyclic, disconnected and multiply-parented trees',()=>{
  for(const change of [
    t=>t.runs.fast.steps[0].values.tree.edges.push({id:99,from:6,to:0}),
    t=>t.runs.fast.steps[0].values.tree.edges.pop(),
    t=>t.runs.fast.steps[0].values.tree.edges.push({id:99,from:2,to:1}),
  ]){const trace=sample('tree');change(trace);assert.throws(()=>T.validate(trace),/tree/);}
});

test('permits empty containers and opaque strings for large integers',()=>{
  const trace=sample('array');
  trace.runs.fast.steps=[{id:0,line:1,label:'empty',stage:'Input',locals:{},values:{a:[],key:'9223372036854775807'}}];
  assert.equal(T.validate(trace),trace);
});

test('rejects numbers that would silently change on JSON export',()=>{
  for(const value of [Infinity,NaN,9007199254740992]){
    const trace=sample('array');trace.runs.fast.steps[0].values.a[0]=value;
    assert.throws(()=>T.validate(trace),/encoded as strings/);
  }
});

test('does not count duplicate compare entries as multiple differences',()=>{
  const trace=sample('knapsack');trace.compare=['dp','dp'];
  assert.throws(()=>T.validate(trace),/unique declared/);
});

test('canonical source carries no generated demo bundle',()=>{
  const embedded=JSON.parse(html.match(/<script id="trace-samples" type="application\/json">([\s\S]*?)<\/script>/)[1]);
  assert.deepEqual(embedded, []);
});

test('an editor hears each new step line once and every clicked line',()=>{
  const posted=[];const link=T.editorLink(m=>posted.push(plain(m)));
  link.step(3);link.step(3);link.step(5);link.reveal(5);link.reveal(5);
  assert.deepEqual(posted,[{type:'step',line:3},{type:'step',line:5},{type:'reveal',line:5},{type:'reveal',line:5}]);
});

test('a re-run keeps the place by line visit and names the first change', () => {
  const run = values => ({steps: values.map((v, i) => ({line: [1, 2, 2, 2, 3][i], label: `s${i}`, values: {total: v}}))});
  const before = run([0, 1, 3, 6, 6]), after = run([0, 2, 6, 12, 12]);
  assert.deepEqual(plain(T.place(before, 2)), {line: 2, visit: 2});
  assert.equal(T.findPlace(after, {line: 2, visit: 2}, 0), 2);
  assert.equal(T.findPlace(after, {line: 2, visit: 9}, 0), 3);
  assert.equal(T.findPlace(after, {line: 7, visit: 1}, 9), 4);
  assert.deepEqual(plain(T.firstChange(before, after)), {index: 1, line: 2, variable: 'total', before: 's1', after: 's1'});
  assert.equal(T.firstChange(before, before), null);
  const shorter = {steps: before.steps.slice(0, 3)};
  assert.equal(T.firstChange(before, shorter).index, 2);
});
