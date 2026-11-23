import { Fragment } from 'react';

// Inline Markdown for generated text: `code` and **bold**, nothing else.
export function InlineMd({ text }: { text: string }) {
  return (
    <>
      {text
        .split(/(`[^`]+`|\*\*[^*]+\*\*)/)
        .map((part, i) =>
          part.startsWith('`') && part.endsWith('`') && part.length > 1 ? (
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
