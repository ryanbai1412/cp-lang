import facts from '@/lib/facts.json';

const block = (code: string, note: string) => (
  <pre className="my-3 overflow-x-auto rounded-md border bg-fd-muted/40 p-3 text-sm">
    <code>{`${code}   # ${note}`}</code>
  </pre>
);

/** The install one-liners, from lib/facts.json, so every page that shows them agrees. */
export function Install() {
  return (
    <>
      {block(facts.install.sh, 'Linux and macOS')}
      {block(facts.install.ps1, 'Windows PowerShell')}
    </>
  );
}

/** As markdown, for llms-full.txt. */
export const installMarkdown = () =>
  `\`\`\`sh\n${facts.install.sh}   # Linux and macOS\n\`\`\`\n\n\`\`\`powershell\n${facts.install.ps1}   # Windows PowerShell\n\`\`\``;
