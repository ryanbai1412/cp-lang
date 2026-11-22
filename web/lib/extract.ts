export interface Extracted {
  name: string;
  version: string;
  source: string;
}

const header = /^\/\/ (\S+): C\+\+17 written by cpl (\S+) from the CP Lang source below\.$/;

export function extractSource(cpp: string): Extracted | null {
  const lines = cpp.replace(/\r\n/g, '\n').split('\n');
  const start = lines.findIndex((l) => header.test(l));
  if (start < 0) return null;
  const [, name, version] = header.exec(lines[start])!;
  const body: string[] = [];
  let i = start + 1;
  while (i < lines.length && lines[i].startsWith('//') && !lines[i].startsWith('//     ') && lines[i] !== '//') i++;
  for (; i < lines.length && lines[i].startsWith('//'); i++) {
    const l = lines[i];
    if (l === '//') body.push('');
    else if (l.startsWith('//     ')) body.push(l.slice(7));
    else break;
  }
  while (body.length && body[0] === '') body.shift();
  while (body.length && body[body.length - 1] === '') body.pop();
  return body.length ? { name, version, source: `${body.join('\n')}\n` } : null;
}
