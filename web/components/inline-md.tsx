import katex from 'katex';
import { Fragment } from 'react';

// Inline Markdown for generated text: `code`, **bold** and math written
// `\( O(n \log n) \)`, rendered by KaTeX at build time. Math KaTeX can't
// render fails the build.
export function InlineMd({ text }: { text: string }) {
  return (
    <>
      {text
        .split(/(\\\([\s\S]+?\\\)|`[^`]+`|\*\*[^*]+\*\*)/)
        .map((part, i) =>
          part.startsWith('\\(') ? (
            <span key={i} dangerouslySetInnerHTML={{ __html: katex.renderToString(part.slice(2, -2).trim(), { throwOnError: true }) }} />
          ) : part.startsWith('`') && part.endsWith('`') && part.length > 1 ? (
            <code key={i}>{part.slice(1, -1)}</code>
          ) : part.startsWith('**') && part.endsWith('**') && part.length > 3 ? (
            <strong key={i}>{part.slice(2, -2)}</strong>
          ) : (
            <Fragment key={i}>{part}</Fragment>
          ),
        )}
    </>
  );
}
