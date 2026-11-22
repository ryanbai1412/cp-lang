import { Badge } from '@/components/ui/badge';

export type Status = 'runs' | 'checks' | 'spec';

const variants = { runs: 'default', checks: 'secondary', spec: 'outline' } as const;
const labels = { runs: 'runs', checks: 'checks', spec: '0.3 spec' };
const titles = {
  runs: 'Runs today in the playground and the compiler',
  checks: 'Passes the type checker; the compiler does not run it yet',
  spec: 'Written to the 0.3 spec; the compiler does not accept it yet',
};

export function StatusBadge({ status }: { status: Status }) {
  return (
    <Badge variant={variants[status]} title={titles[status]} className="font-mono text-[11px]">
      {labels[status]}
    </Badge>
  );
}
