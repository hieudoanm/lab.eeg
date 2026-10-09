import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { StatisticsSummary, StatisticsTable } from '../statistics-table';
import type { ChannelStatistics } from '@/lib/api';

const channel = (overrides: Partial<ChannelStatistics> = {}): ChannelStatistics => ({
  channel_index: 0,
  channel_name: 'Fp1',
  unit: 'µV',
  mean: 1.5,
  standard_deviation: 0.25,
  minimum: -2,
  maximum: 3,
  peak_to_peak: 5,
  valid_sample_count: 1000,
  ...overrides,
});

describe('StatisticsTable', () => {
  it('renders an empty state when there are no channels', () => {
    render(<StatisticsTable channels={[]} unit={null} />);
    expect(screen.getByText('No channel statistics available.')).toBeInTheDocument();
  });

  it('renders each channel header and its statistics', () => {
    render(<StatisticsTable channels={[channel(), channel({ channel_index: 1 })]} unit='µV' />);

    expect(screen.getByText('Ch 0')).toBeInTheDocument();
    expect(screen.getByText('Ch 1')).toBeInTheDocument();
    expect(screen.getAllByText('Fp1')).toHaveLength(2);

    expect(screen.getAllByText('Mean')).toHaveLength(2);
    expect(screen.getAllByText('Std. deviation')).toHaveLength(2);
    expect(screen.getAllByText('Minimum')).toHaveLength(2);
    expect(screen.getAllByText('Maximum')).toHaveLength(2);
    expect(screen.getAllByText('Peak-to-peak')).toHaveLength(2);
    expect(screen.getAllByText('Valid samples')).toHaveLength(2);

    expect(screen.getAllByText('1.5')).toHaveLength(2);
    expect(screen.getAllByText('1,000')).toHaveLength(2);
  });

  it('formats null values with an em dash', () => {
    render(
      <StatisticsTable
        channels={[channel({ mean: null, standard_deviation: null, peak_to_peak: null })]}
        unit={null}
      />,
    );
    expect(screen.getAllByText('—')).toHaveLength(3);
  });

  it('omits the unit annotation when no unit is provided', () => {
    render(<StatisticsTable channels={[channel({ unit: null })]} unit={null} />);
    expect(screen.queryByText('(µV)')).not.toBeInTheDocument();
  });
});

describe('StatisticsSummary', () => {
  it('summarises channel and sample counts', () => {
    render(
      <StatisticsSummary
        channels={[
          channel({ channel_index: 0, valid_sample_count: 1000 }),
          channel({ channel_index: 1, valid_sample_count: 500 }),
        ]}
      />,
    );
    expect(screen.getByText('Channels analysed')).toBeInTheDocument();
    expect(screen.getByText('2')).toBeInTheDocument();
    expect(screen.getByText('Total valid samples')).toBeInTheDocument();
    expect(screen.getByText('1,500')).toBeInTheDocument();
  });

  it('handles an empty channel list', () => {
    render(<StatisticsSummary channels={[]} />);
    expect(screen.getAllByText('0')).toHaveLength(2);
  });
});
