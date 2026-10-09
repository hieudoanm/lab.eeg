import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const plotly = vi.hoisted(() => ({
  newPlot: vi.fn().mockResolvedValue(undefined),
  react: vi.fn().mockResolvedValue(undefined),
  purge: vi.fn(),
  Plots: { resize: vi.fn() },
}));

vi.mock('plotly.js-dist', () => ({ default: plotly }));

import { ChartContainer } from '../../molecules/chart-container';
import type { ChannelStatistics } from '@/lib/api';
import type { TimeSeriesStats } from '../time-series-chart';

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

const baseProps = {
  channelName: 'Fp1',
  unit: 'µV',
  data: [0, 1, 2, 3],
  samplingFrequencyHz: 250,
};

beforeEach(() => {
  vi.clearAllMocks();
});

describe('ChartContainer', () => {
  it('renders a loading state while data is pending', () => {
    render(<ChartContainer {...baseProps} isLoading />);
    expect(screen.getByText('Channel: Fp1')).toBeInTheDocument();
    expect(screen.getByRole('status')).toHaveTextContent('Loading time-series data…');
  });

  it('renders an error state when an error is provided', () => {
    render(<ChartContainer {...baseProps} error='Network unavailable' />);
    expect(screen.getByText('Channel: Fp1')).toBeInTheDocument();
    expect(screen.getByRole('alert')).toHaveTextContent('Network unavailable');
  });

  it('renders the chart and computed statistics when ready', () => {
    render(<ChartContainer {...baseProps} />);
    expect(screen.getByText('Channel: Fp1')).toBeInTheDocument();
    expect(screen.getByText('Sampling frequency: 250 Hz')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Show data table' })).toBeInTheDocument();
    expect(screen.getByText('Mean')).toBeInTheDocument();
    expect(screen.getByText('1.500')).toBeInTheDocument();
    expect(plotly.newPlot).toHaveBeenCalled();
  });

  it('prefers provided statistics over computed ones', () => {
    const stats: TimeSeriesStats = {
      mean: 9.999,
      std: 8.888,
      min: 7.777,
      max: 6.666,
      peakToPeak: 5.555,
      validSampleCount: 42,
    };
    render(<ChartContainer {...baseProps} data={[0, 0, 0]} stats={stats} />);
    expect(screen.getByText('9.999')).toBeInTheDocument();
    expect(screen.getByText('42')).toBeInTheDocument();
  });

  it('toggles to an accessible data table and back', async () => {
    const user = userEvent.setup();
    render(<ChartContainer {...baseProps} />);

    await user.click(screen.getByRole('button', { name: 'Show data table' }));

    const table = within(screen.getByRole('table'));
    expect(table.getByRole('columnheader', { name: 'Sample' })).toBeInTheDocument();
    expect(table.getByRole('columnheader', { name: 'Value' })).toBeInTheDocument();
    expect(table.getByText('0.000')).toBeInTheDocument();
    expect(table.getByText('3.000')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Show chart' }));
    expect(screen.getByRole('button', { name: 'Show data table' })).toBeInTheDocument();
  });

  it('notes when only the first maxPoints samples are shown', async () => {
    const user = userEvent.setup();
    render(<ChartContainer {...baseProps} data={[1, 2, 3]} maxPoints={2} />);

    await user.click(screen.getByRole('button', { name: 'Show data table' }));

    const table = within(screen.getByRole('table'));
    expect(table.getByText('Showing first 2 of 3 samples')).toBeInTheDocument();
    expect(table.queryByText('3.000')).not.toBeInTheDocument();
  });

  it('renders the all-channels summary when multiple channels are supplied', () => {
    render(
      <ChartContainer
        {...baseProps}
        allChannels={[channel(), channel({ channel_index: 1, valid_sample_count: 500 })]}
      />,
    );

    expect(screen.getByText('All channels summary')).toBeInTheDocument();
    expect(screen.getByText('Channels analysed')).toBeInTheDocument();
    expect(screen.getByText('Total valid samples')).toBeInTheDocument();
    expect(screen.getByText('1,500')).toBeInTheDocument();
  });

  it('hides the all-channels summary for a single channel', () => {
    render(<ChartContainer {...baseProps} allChannels={[channel()]} />);
    expect(screen.queryByText('All channels summary')).not.toBeInTheDocument();
  });
});
