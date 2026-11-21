import * as assert from 'node:assert/strict';
import { test } from 'node:test';
import { fileSamples, parts, planLenses } from '../../src/samples';

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
    '// Count something.',
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
    const samples = fileSamples('d', ['i1'], [], []);
    const lenses = planLenses(PROGRAM, samples).map(l => [l.line, l.title, l.command]);
    assert.deepEqual(lenses, [
        [0, '$(play) Run on i1', 'cp.runSample'],
        [0, 'Stress', 'cp.stress'],
        [0, 'Trace', 'cp.trace'],
        [3, 'Stress against count_brute', 'cp.stress'],
        [3, 'Trace', 'cp.trace'],
    ]);
    assert.deepEqual(planLenses(PROGRAM, samples)[3].args, ['count']);
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
