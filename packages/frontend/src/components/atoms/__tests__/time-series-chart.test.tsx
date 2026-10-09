import { render } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const plotly = vi.hoisted(() => ({
  newPlot: vi.fn().mockResolvedValue(undefined),
  react: vi.fn().mockResolvedValue(undefined),
  purge: vi.fn(),
  Plots: { resize: vi.fn() },
}));

vi.mock('plotly.js-dist', () => ({ default: plotly }));

import { computeTimeSeriesStats, TimeSeriesChart } from '../time-series-chart';

interface Trace {
  x: number[];
  y: number[];
}
interface Layout {
  height: number;
  yaxis: { title: string };
}

const lastCall = () => {
  const call = plotly.newPlot.mock.calls.at(-1);
  if (!call) throw new Error('Plotly.newPlot was not called');
  return call as unknown as [HTMLElement, [Trace], Layout, unknown];
};

beforeEach(() => {
  vi.clearAllMocks();
});

describe('TimeSeriesChart', () => {
  it('renders the data using Plotly.newPlot', () => {
    render(
      <TimeSeriesChart channelName='Fp1' unit='µV' data={[1, 2, 3, 4]} samplingFrequencyHz={100} />,
    );

    expect(plotly.newPlot).toHaveBeenCalledTimes(1);
    const [, traces, layout] = lastCall();
    expect(traces).toHaveLength(1);
    expect(traces[0].y).toEqual([1, 2, 3, 4]);
    expect(layout.yaxis.title).toEqual({ text: 'µV' });
  });

  it('builds a time axis from the sampling frequency', () => {
    render(
      <TimeSeriesChart channelName='Fp1' unit={null} data={[1, 2, 3]} samplingFrequencyHz={10} />,
    );

    const [, traces, layout] = lastCall();
    expect(traces[0].x).toEqual([0, 0.1, 0.2]);
    expect(layout.yaxis.title).toEqual({ text: 'Amplitude' });
  });

  it('applies the requested height', () => {
    render(
      <TimeSeriesChart
        channelName='Fp1'
        unit={null}
        data={[1]}
        samplingFrequencyHz={1}
        height={420}
      />,
    );
    expect(lastCall()[2].height).toBe(420);
  });

  it('downsamples data that exceeds maxPoints', () => {
    const data = Array.from({ length: 250 }, (_, i) => i);
    render(
      <TimeSeriesChart
        channelName='Fp1'
        unit={null}
        data={data}
        samplingFrequencyHz={100}
        maxPoints={100}
      />,
    );

    const [, traces] = lastCall();
    expect(traces[0].y).toHaveLength(84);
    expect(traces[0].x).toHaveLength(84);
  });

  it('keeps all points when within maxPoints', () => {
    const data = Array.from({ length: 50 }, (_, i) => i);
    render(
      <TimeSeriesChart
        channelName='Fp1'
        unit={null}
        data={data}
        samplingFrequencyHz={100}
        maxPoints={100}
      />,
    );
    expect(lastCall()[1][0].y).toHaveLength(50);
  });

  it('purges the plot on unmount', () => {
    const { unmount } = render(
      <TimeSeriesChart channelName='Fp1' unit={null} data={[1]} samplingFrequencyHz={1} />,
    );
    unmount();
    expect(plotly.purge).toHaveBeenCalledTimes(1);
  });
});

describe('computeTimeSeriesStats', () => {
  it('computes statistics for finite samples', () => {
    const stats = computeTimeSeriesStats([1, 2, 3, 4]);
    expect(stats.mean).toBeCloseTo(2.5);
    expect(stats.std).toBeCloseTo(Math.sqrt(1.25));
    expect(stats.min).toBe(1);
    expect(stats.max).toBe(4);
    expect(stats.peakToPeak).toBe(3);
    expect(stats.validSampleCount).toBe(4);
  });

  it('excludes non-finite samples', () => {
    const stats = computeTimeSeriesStats([1, 2, 3, Number.NaN, Number.POSITIVE_INFINITY]);
    expect(stats.validSampleCount).toBe(3);
    expect(stats.mean).toBe(2);
    expect(stats.min).toBe(1);
    expect(stats.max).toBe(3);
  });

  it('returns zeros when there are no valid samples', () => {
    expect(computeTimeSeriesStats([])).toEqual({
      mean: 0,
      std: 0,
      min: 0,
      max: 0,
      peakToPeak: 0,
      validSampleCount: 0,
    });
    expect(computeTimeSeriesStats([Number.NaN]).validSampleCount).toBe(0);
  });
});
