import * as assert from 'node:assert/strict';
import { test } from 'node:test';
import { parts, replayLenses } from '../../src/samples';

const PROGRAM = [
    '// Count something.',
    'let n = read()',
    '',
    '@memo',
    '@alt(count_brute)',
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
        { name: 'count', line: 3, fnLine: 5, decorators: [{ name: 'memo', args: undefined }, { name: 'alt', args: 'count_brute' }] },
        { name: 'count_brute', line: 9, fnLine: 9, decorators: [] },
        { name: 'helper', line: 10, fnLine: 10, decorators: [{ name: 'grader', args: undefined }] },
        { name: 'api', line: 13, fnLine: 13, decorators: [] },
    ]);
});

test('replayLenses puts Replay above each fn of the solution, not its tools', () => {
    const text = ['fn most() -> Int:', '    return 1', '', '@alt(program)', 'fn slow():', '    print(1)', 'print(most())'].join('\n');
    assert.deepEqual(
        replayLenses(text).map(l => [l.line, l.title, l.command, l.args]),
        [[0, '$(debug-restart) Replay this call', 'cp.replayCall', [{ fn: 'most' }]]],
    );
});
