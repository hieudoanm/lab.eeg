'use client';

import { useState } from 'react';
import {
  TimeSeriesChart,
  computeTimeSeriesStats,
  type TimeSeriesStats,
} from '../atoms/time-series-chart';
import { StatisticsTable, StatisticsSummary } from '../atoms/statistics-table';
import { LoadingState, ErrorState } from '@/components/atoms/states';
import { Card } from '@/components/atoms/card';
import type { ChannelStatistics } from '@/lib/api';

export interface ChartContainerProps {
  channelName: string;
  unit: string | null;
  data: number[];
  samplingFrequencyHz: number;
  stats?: TimeSeriesStats;
  allChannels?: ChannelStatistics[];
  maxPoints?: number;
  height?: number;
  error?: string;
  isLoading?: boolean;
}

export const ChartContainer = ({
  channelName,
  unit,
  data,
  samplingFrequencyHz,
  stats,
  allChannels,
  maxPoints = 5000,
  height = 300,
  error,
  isLoading = false,
}: ChartContainerProps) => {
  const [showTable, setShowTable] = useState(false);
  const computedStats = stats ?? computeTimeSeriesStats(data);

  if (isLoading) {
    return (
      <Card title={`Channel: ${channelName}`}>
        <LoadingState label='Loading time-series data…' />
      </Card>
    );
  }

  if (error) {
    return (
      <Card title={`Channel: ${channelName}`}>
        <ErrorState message={error} />
      </Card>
    );
  }

  return (
    <Card title={`Channel: ${channelName}`}>
      <div className='flex flex-col gap-4'>
        <div className='flex items-center justify-between'>
          <span className='text-base-content/60 text-sm'>
            Sampling frequency: {samplingFrequencyHz} Hz
          </span>
          <button
            type='button'
            onClick={() => setShowTable(!showTable)}
            className='link link-primary text-sm'>
            {showTable ? 'Show chart' : 'Show data table'}
          </button>
        </div>

        {showTable ? (
          <div className='overflow-x-auto'>
            <table className='table-sm table'>
              <thead>
                <tr className='border-base-300 border-b'>
                  <th className='p-2 text-left'>Sample</th>
                  <th className='p-2 text-right'>Value</th>
                </tr>
              </thead>
              <tbody>
                {data.slice(0, maxPoints).map((value, index) => (
                  <tr key={index} className='border-base-300 border-b last:border-0'>
                    <td className='p-2 font-mono'>{index}</td>
                    <td className='p-2 text-right font-mono'>{value.toFixed(3)}</td>
                  </tr>
                ))}
                {data.length > maxPoints && (
                  <tr>
                    <td colSpan={2} className='text-base-content/60 p-2 text-center'>
                      Showing first {maxPoints} of {data.length} samples
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        ) : (
          <TimeSeriesChart
            channelName={channelName}
            unit={unit}
            data={data}
            samplingFrequencyHz={samplingFrequencyHz}
            maxPoints={maxPoints}
            height={height}
          />
        )}

        <div className='border-base-300 border-t pt-4'>
          <MetadataGrid
            columns={3}
            items={[
              { label: 'Mean', value: computedStats.mean.toFixed(3) },
              { label: 'Std. deviation', value: computedStats.std.toFixed(3) },
              { label: 'Min', value: computedStats.min.toFixed(3) },
              { label: 'Max', value: computedStats.max.toFixed(3) },
              { label: 'Peak-to-peak', value: computedStats.peakToPeak.toFixed(3) },
              { label: 'Valid samples', value: computedStats.validSampleCount.toLocaleString() },
            ]}
          />
        </div>

        {allChannels && allChannels.length > 1 && (
          <div className='border-base-300 border-t pt-4'>
            <h5 className='mb-3 font-medium'>All channels summary</h5>
            <StatisticsSummary channels={allChannels} />
            <StatisticsTable channels={allChannels} unit={unit} />
          </div>
        )}
      </div>
    </Card>
  );
};

interface MetadataGridProps {
  columns: number;
  items: Array<{ label: string; value: string | React.ReactNode }>;
}

const MetadataGrid = ({ columns, items }: MetadataGridProps) => {
  return (
    <dl className='grid gap-4' style={{ gridTemplateColumns: `repeat(${columns}, 1fr)` }}>
      {items.map((item, index) => (
        <div key={index} className='flex flex-col gap-1'>
          <dt className='text-base-content/60 text-xs'>{item.label}</dt>
          <dd className='font-mono text-sm'>{item.value}</dd>
        </div>
      ))}
    </dl>
  );
};
