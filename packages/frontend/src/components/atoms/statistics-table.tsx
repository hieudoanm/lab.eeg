'use client';

import { MetadataGrid } from '@/components/atoms/metadata-grid';
import type { ChannelStatistics } from '@/lib/api';

export interface StatisticsTableProps {
  channels: ChannelStatistics[];
  unit: string | null;
}

export interface StatisticsSummaryProps {
  channels: ChannelStatistics[];
}

const STAT_ROWS = [
  { key: 'mean', label: 'Mean' },
  { key: 'standard_deviation', label: 'Std. deviation' },
  { key: 'minimum', label: 'Minimum' },
  { key: 'maximum', label: 'Maximum' },
  { key: 'peak_to_peak', label: 'Peak-to-peak' },
  { key: 'valid_sample_count', label: 'Valid samples' },
] as const;

const formatValue = (value: number | string | null | undefined, isCount = false): string => {
  if (value === null || value === undefined) return '—';
  const num = typeof value === 'string' ? Number(value) : value;
  if (isCount) return num.toLocaleString();
  return num.toLocaleString(undefined, { maximumFractionDigits: 3 });
};

export const StatisticsTable = ({ channels, unit }: StatisticsTableProps) => {
  if (channels.length === 0) {
    return (
      <div className='text-base-content/60 py-8 text-center'>No channel statistics available.</div>
    );
  }

  return (
    <div className='space-y-4'>
      {channels.map((channel) => (
        <div key={channel.channel_index} className='rounded-box border-base-300 border p-4'>
          <h4 className='mb-3 flex items-center gap-2 font-medium'>
            <span className='text-base-content/60 font-mono text-sm'>
              Ch {channel.channel_index}
            </span>
            {channel.channel_name}
            {unit && <span className='text-base-content/60 text-xs'>({unit})</span>}
          </h4>
          <MetadataGrid
            columns={3}
            items={STAT_ROWS.map((stat) => ({
              label: stat.label,
              value: formatValue(
                channel[stat.key as keyof ChannelStatistics],
                stat.key === 'valid_sample_count',
              ),
            }))}
          />
        </div>
      ))}
    </div>
  );
};

export const StatisticsSummary = ({ channels }: StatisticsSummaryProps) => {
  const totalChannels = channels.length;
  const totalValidSamples = channels.reduce((sum, ch) => sum + (ch.valid_sample_count ?? 0), 0);

  return (
    <MetadataGrid
      columns={2}
      items={[
        { label: 'Channels analysed', value: String(totalChannels) },
        { label: 'Total valid samples', value: totalValidSamples.toLocaleString() },
      ]}
    />
  );
};
