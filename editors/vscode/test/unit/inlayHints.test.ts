import * as assert from 'node:assert/strict';
import { test } from 'node:test';
import { computeInlayHints } from '../../src/inlayHintsCore';

test('computeInlayHints infers v2.1 container constructors, compress, and domain loops', () => {
    const source = [
        'domain V = 1..=n',
        'let xs = compress(vals)',
        'var st = LazySegTree<V, Int>(AddSum, a)',
        'let t = Rooted<V>(g, V.first)',
        'var seen = Marks<V>()',
        'let g = Graph.undirected(V, edges)',
        'var annotated: Dsu<V> = Dsu<V>()',
        'for v in V:',
        '    for mut u in V:',
        '        print(u)',
        '// let ignored = Dsu<V>()',
    ].join('\n');

    assert.deepEqual(
        computeInlayHints(source).map(h => [h.line, h.character, h.label]),
        [
            [1, 6, ': Compressed'],
            [2, 6, ': LazySegTree<V, Int>'],
            [3, 5, ': Rooted<V>'],
            [4, 8, ': Marks<V>'],
            [5, 5, ': Graph'],
            [7, 5, ': V'],
            [8, 13, ': V'],
        ],
    );
});

test('computeInlayHints skips loops over undeclared capitalized identifiers', () => {
    const source = ['for x in Unknown:', '    let n = read()'].join('\n');
    assert.deepEqual(computeInlayHints(source), []);
});
