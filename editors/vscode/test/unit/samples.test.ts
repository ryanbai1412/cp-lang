import * as assert from 'node:assert/strict';
import { test } from 'node:test';
import { fileSamples, inlineSamples, parts, planLenses, readString } from '../../src/samples';

test('readString decodes escapes and doubled braces', () => {
    assert.deepEqual(readString('"3\\n1 2 3\\n"', 0), { value: '3\n1 2 3\n', end: 12 });
    assert.equal(readString('"a{{b}}\\"c"', 0)?.value, 'a{b}"c');
    assert.equal(readString('r"a\\n{{"', 0)?.value, 'a\\n{{');
    assert.equal(readString('"never closed', 0), undefined);
    assert.equal(readString('x', 0), undefined);
});

test('inlineSamples finds each @sample with its line', () => {
    const text = ['// header', '@sample("3\\n1 2 3\\n", "6\\n")', 'let n = read()', '  @sample("1\\n")', '@sample(input)'].join('\n');
    assert.deepEqual(inlineSamples(text), [
        { label: 'sample 1', input: { text: '3\n1 2 3\n' }, expected: { text: '6\n' }, line: 1 },
        { label: 'sample 2', input: { text: '1\n' }, line: 3 },
    ]);
});

test('fileSamples orders scratch inputs and test folders by number', () => {
    const found = fileSamples('d', ['d.cp', 'i10', 'i2', 'o2', 'notes'], ['2.in', '1.in', '1.out', 'hand-1.in'], ['1.in']);
    assert.deepEqual(
        found.map(s => [s.label, s.input, s.expected]),
        [
            ['i2', { file: 'i2' }, { file: 'o2' }],
            ['i10', { file: 'i10' }, undefined],
            ['1', { file: 'd.tests/1.in' }, { file: 'd.tests/1.out' }],
            ['hand-1', { file: 'd.tests/hand-1.in' }, undefined],
            ['2', { file: 'd.tests/2.in' }, undefined],
            ['1', { file: 'd.samples/1.in' }, undefined],
        ],
    );
});

const PROGRAM = [
    '@sample("2\\n7 7\\n", "1\\n")',
    'let n = read()',
    '',
    '@memo',
    '@brute(count_brute)',
    'fn count() -> Int:',
    '    fn inner() -> Int = 1',
    '    inner()',
    '',
    'fn count_brute() -> Int = 0',
    '@grader fn helper(g: Graph<V>) = print(g)',
    '@multitest',
    'let x = 1',
    'pub fn api() = 2',
].join('\n');

test('parts lists top-level fns with their decorators', () => {
    assert.deepEqual(parts(PROGRAM), [
        { name: 'count', line: 3, fnLine: 5, decorators: [{ name: 'memo', args: undefined }, { name: 'brute', args: 'count_brute' }] },
        { name: 'count_brute', line: 9, fnLine: 9, decorators: [] },
        { name: 'helper', line: 10, fnLine: 10, decorators: [{ name: 'grader', args: undefined }] },
        { name: 'api', line: 13, fnLine: 13, decorators: [] },
    ]);
});

test('planLenses puts Run above the file and Stress above each bruted part', () => {
    const samples = inlineSamples(PROGRAM);
    const lenses = planLenses(PROGRAM, samples).map(l => [l.line, l.title, l.command]);
    assert.deepEqual(lenses, [
        [0, '$(play) Run on sample 1', 'cp.runSample'],
        [0, 'Stress', 'cp.stress'],
        [0, 'Trace', 'cp.trace'],
        [0, '$(play) Run this sample', 'cp.runSample'],
        [3, 'Stress against count_brute', 'cp.stress'],
        [3, 'Trace', 'cp.trace'],
    ]);
    assert.deepEqual(planLenses(PROGRAM, samples)[4].args, ['count']);
});

test('planLenses supports v2.1 @brute(main), @checker and @interactor lenses', () => {
    const v21 = [
        'let n = read()',
        '',
        '@brute(main)',
        'fn slow():',
        '    print(n)',
        '',
        '@checker',
        'fn check():',
        '    assert true',
        '',
        '@interactor',
        'fn judge():',
        '    print(0)',
    ].join('\n');
    assert.deepEqual(
        planLenses(v21, []).map(l => [l.line, l.title, l.command]),
        [
            [0, '$(play) Run', 'cp.runSample'],
            [0, 'Stress', 'cp.stress'],
            [0, 'Trace', 'cp.trace'],
            [2, 'Stress main against slow', 'cp.stress'],
            [2, 'Trace', 'cp.trace'],
            [6, 'Stress with checker check', 'cp.stress'],
            [10, 'Stress with interactor judge', 'cp.stress'],
        ],
    );
});

test('planLenses on a file with no samples and no brute still offers Run and Trace', () => {
    assert.deepEqual(
        planLenses('print(1)\n', []).map(l => l.title),
        ['$(play) Run', 'Trace'],
    );
    const two = fileSamples('d', ['i1', 'i2'], [], []);
    assert.deepEqual(
        planLenses('print(1)\n', two).map(l => l.title),
        ['$(play) Run on i1', 'Run on all 2 samples', 'Trace'],
    );
});
