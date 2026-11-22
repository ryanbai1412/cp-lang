import { Badge } from '@/components/ui/badge';

export type Status = 'runs' | 'checks';

const variants = { runs: 'default', checks: 'secondary' } as const;
const titles = {
  runs: 'Runs today in the playground and the compiler',
  checks: 'Passes the type checker; the compiler does not run it yet',
};

export function StatusBadge({ status }: { status: Status }) {
  return (
    <Badge variant={variants[status]} title={titles[status]} className="font-mono text-[11px]">
      {status}
    </Badge>
  );
}
