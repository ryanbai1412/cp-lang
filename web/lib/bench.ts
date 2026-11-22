import { readFileSync } from 'node:fs';
import { join } from 'node:path';

export interface Dist {
  n: number;
  median: number;
  p90: number;
  p95: number;
  p99: number;
  worst: number;
}

export interface Port {
  problem: string;
  port: string;
  emitted_bytes: number;
  input?: string;
  orig_s?: number;
  port_s?: number;
  orig_rss_kb?: number;
  port_rss_kb?: number;
  time_ratio?: number;
  rss_ratio?: number;
  above_floor?: boolean;
  skip?: string;
}

export interface Bench {
  meta: { commit: string; cpl_version: string; gxx: string; runs: number; problems: number; cpu: string; cores: number };
  summary: {
    measured: number;
    above_floor: number;
    floor_ms: number;
    max_emitted_bytes: number;
    emitted_files: number;
    largest_emitted_port: string;
    time_above_floor: Dist;
    rss_above_floor: Dist;
    target_met: boolean;
  };
  ports: Port[];
}

export function bench(): Bench {
  return JSON.parse(readFileSync(join(process.cwd(), '../bench/corpus-p95-results.json'), 'utf8'));
}

export function histogram(values: number[], edges: number[]) {
  return edges.slice(0, -1).map((lo, i) => {
    const hi = edges[i + 1];
    return { lo, hi, count: values.filter((v) => v >= lo && (v < hi || (i === edges.length - 2 && v >= hi))).length };
  });
}
