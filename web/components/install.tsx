import facts from '@/lib/facts.json';

const block = (code: string, note: string) => (
  <pre tabIndex={0} className="my-3 overflow-x-auto rounded-md border bg-fd-muted/40 p-3 text-sm">
    <code>{`${code}   # ${note}`}</code>
  </pre>
);

/** What each release channel is, shared by every page that offers both. */
export const channelNotes = {
  edge: 'the newest commit of main that passed every check, and the version these docs describe',
  stable: 'beta while CP Lang is 0.x; until the first stable release it installs edge',
};

/** The install one-liners for both channels, from lib/facts.json, so every page that shows them agrees. */
export function Install() {
  return (
    <>
      <p>
        <strong>Edge</strong>: {channelNotes.edge}.
      </p>
      {block(facts.install.edge.sh, 'Linux and macOS')}
      {block(facts.install.edge.ps1, 'Windows PowerShell')}
      <p>
        <strong>Stable</strong> ({channelNotes.stable}):
      </p>
      {block(facts.install.stable.sh, 'Linux and macOS')}
      {block(facts.install.stable.ps1, 'Windows PowerShell')}
    </>
  );
}

const md = (shell: string, code: string, note: string) => `\`\`\`${shell}\n${code}   # ${note}\n\`\`\``;

/** As markdown, for llms-full.txt. */
export const installMarkdown = () =>
  [
    `Edge: ${channelNotes.edge}.`,
    md('sh', facts.install.edge.sh, 'Linux and macOS'),
    md('powershell', facts.install.edge.ps1, 'Windows PowerShell'),
    `Stable (${channelNotes.stable}):`,
    md('sh', facts.install.stable.sh, 'Linux and macOS'),
    md('powershell', facts.install.stable.ps1, 'Windows PowerShell'),
  ].join('\n\n');
