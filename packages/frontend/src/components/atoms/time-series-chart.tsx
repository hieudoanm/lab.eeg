'use client';

import { useMemo, useRef, useEffect } from 'react';
import Plotly from 'plotly.js-dist';

export interface TimeSeriesChartProps {
  channelName: string;
  unit: string | null;
  data: number[];
  samplingFrequencyHz: number;
  maxPoints?: number;
  height?: number;
}

const DEFAULT_MAX_POINTS = 5000;
const DEFAULT_HEIGHT = 300;

const downsample = (data: number[], maxPoints: number): number[] => {
  if (data.length <= maxPoints) return data;

  const step = Math.ceil(data.length / maxPoints);
  const result: number[] = [];
  for (let i = 0; i < data.length; i += step) {
    result.push(data[i]);
  }
  return result;
};

const generateTimeAxis = (dataLength: number, samplingFrequencyHz: number): number[] => {
  const dt = 1 / samplingFrequencyHz;
  return Array.from({ length: dataLength }, (_, i) => i * dt);
};

export const TimeSeriesChart = ({
  channelName,
  unit,
  data,
  samplingFrequencyHz,
  maxPoints = DEFAULT_MAX_POINTS,
  height = DEFAULT_HEIGHT,
}: TimeSeriesChartProps) => {
  const containerRef = useRef<HTMLDivElement>(null);

  const downsampledData = useMemo(() => downsample(data, maxPoints), [data, maxPoints]);
  const timeAxis = useMemo(
    () => generateTimeAxis(downsampledData.length, samplingFrequencyHz),
    [downsampledData.length, samplingFrequencyHz],
  );

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const trace = {
      x: timeAxis,
      y: downsampledData,
      type: 'scattergl' as const,
      mode: 'lines' as const,
      line: { color: '#3b82f6', width: 1 },
      hovertemplate: `%{y:.3f} %{xaxis.hoverformat}<extra>${channelName}</extra>`,
    };

    const layout = {
      margin: { t: 10, r: 10, b: 40, l: 50 },
      height,
      xaxis: {
        title: { text: 'Time (s)' },
        hoverformat: '.3f',
        showgrid: true,
        gridcolor: 'rgba(128,128,128,0.2)',
      },
      yaxis: {
        title: { text: unit ?? 'Amplitude' },
        showgrid: true,
        gridcolor: 'rgba(128,128,128,0.2)',
      },
      hovermode: 'x unified' as const,
      dragmode: 'pan' as const,
    };

    Plotly.newPlot(container, [trace], layout, {
      displayModeBar: true,
      modeBarButtonsToRemove: [
        'lasso2d',
        'select2d',
        'autoScale2d',
        'hoverClosestCartesian',
        'hoverCompareCartesian',
      ],
      responsive: true,
    });

    return () => {
      Plotly.purge(container);
    };
  }, [channelName, downsampledData, timeAxis, unit, height]);

  return <div ref={containerRef} className='w-full' />;
};

export interface TimeSeriesStats {
  mean: number;
  std: number;
  min: number;
  max: number;
  peakToPeak: number;
  validSampleCount: number;
}

export const computeTimeSeriesStats = (data: number[]): TimeSeriesStats => {
  const valid = data.filter((v) => Number.isFinite(v));
  const count = valid.length;
  if (count === 0) {
    return { mean: 0, std: 0, min: 0, max: 0, peakToPeak: 0, validSampleCount: 0 };
  }
  const mean = valid.reduce((a, b) => a + b, 0) / count;
  const variance = valid.reduce((a, b) => a + (b - mean) ** 2, 0) / count;
  const std = Math.sqrt(variance);
  const min = Math.min(...valid);
  const max = Math.max(...valid);
  return { mean, std, min, max, peakToPeak: max - min, validSampleCount: count };
};
