import { Badge } from '@/components/ui/badge';

export type Status = 'runs' | 'checks' | 'spec' | 'rejects';

const variants = { runs: 'default', checks: 'secondary', spec: 'outline', rejects: 'destructive' } as const;
const labels = { runs: 'runs', checks: 'checks', spec: '0.3 spec', rejects: 'error on purpose' };
const titles = {
  runs: 'Runs today in the playground and the compiler',
  checks: 'Passes the type checker; the compiler does not run it yet',
  spec: 'Written to the 0.3 spec; the compiler does not accept it yet',
  rejects: 'A mistake the compiler rejects: run it to see the error, then fix it',
};

export function StatusBadge({ status }: { status: Status }) {
  return (
    <Badge variant={variants[status]} title={titles[status]} className="font-mono text-[11px]">
      {labels[status]}
    </Badge>
  );
}
