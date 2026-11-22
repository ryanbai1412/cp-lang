import { Badge } from '@/components/ui/badge';

export type Status = 'runs' | 'checks' | 'spec';

const variants = { runs: 'default', checks: 'secondary', spec: 'outline' } as const;
const titles = {
  runs: 'Runs today in the playground and the compiler',
  checks: 'Type-checks against the 0.3 spec; does not run yet',
  spec: 'Designed in the spec; not implemented yet',
};

export function StatusBadge({ status }: { status: Status }) {
  return (
    <Badge variant={variants[status]} title={titles[status]} className="font-mono text-[11px]">
      {status}
    </Badge>
  );
}
