import { describe, expect, it } from 'vitest';
import { extractSource } from './extract';

const cpp = `// optionals.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
// For judge \`generic\`: g++ -std=c++17 -O2; 8 MB stack.
//
//     let n: 1..=2e5 = read()
//
//     for x in a:
//         print(x)

#include <cstdio>
int main() {}
`;

describe('extractSource', () => {
  it('recovers the quoted source, keeping blank lines and indentation', () => {
    expect(extractSource(cpp)).toEqual({
      name: 'optionals.cp',
      version: '0.2.2',
      source: 'let n: 1..=2e5 = read()\n\nfor x in a:\n    print(x)\n',
    });
  });
  it('returns null for C++ that cpl did not write', () => {
    expect(extractSource('#include <bits/stdc++.h>\nint main() {}\n')).toBeNull();
  });
});
