'use client';

import { useState, useCallback } from 'react';
import { useChannelSelection, ChannelSelector } from '@/components/molecules/channel-selector';
import { TimeSeriesChart } from '@/components/atoms/time-series-chart';
import { LoadingState, ErrorState } from '@/components/atoms/states';
import { api } from '@/trpc/react';

export interface SignalVisualisationProps {
  datasetId: string;
  samplingFrequencyHz: number;
  channelNames: string[];
  channelUnits: (string | null)[];
  durationSeconds: number | null;
}

const DEFAULT_WINDOW_SECONDS = 10;

export const SignalVisualisation = ({
  datasetId,
  samplingFrequencyHz,
  channelNames,
  channelUnits,
  durationSeconds,
}: SignalVisualisationProps) => {
  const [startSeconds, setStartSeconds] = useState(0);
  const [endSeconds, setEndSeconds] = useState(() =>
    Math.min(DEFAULT_WINDOW_SECONDS, durationSeconds ?? DEFAULT_WINDOW_SECONDS),
  );
  const [selectedChannelIndex, setSelectedChannelIndex] = useState(0);

  const channels = channelNames.map((name, index) => ({
    index,
    name: name || `Channel ${index}`,
  }));

  const {
    selectedIndex,
    nextChannel,
    prevChannel,
    channelCount: totalChannels,
  } = useChannelSelection(channels, selectedChannelIndex);

  const {
    data: windowData,
    isLoading,
    error,
  } = api.signals.window.useQuery(
    {
      datasetId,
      startSeconds,
      endSeconds,
      channels: [selectedIndex],
    },
    {
      enabled: true,
      retry: 1,
    },
  );

  const handleTimeRangeChange = useCallback((newStart: number, newEnd: number) => {
    setStartSeconds(newStart);
    setEndSeconds(newEnd);
  }, []);

  const handleChannelChange = useCallback((index: number) => {
    setSelectedChannelIndex(index);
  }, []);

  if (isLoading && !windowData) {
    return <LoadingState label='Loading signal data…' />;
  }

  if (error && !windowData) {
    return <ErrorState message={error.message} />;
  }

  if (!windowData) {
    return <LoadingState label='Loading signal data…' />;
  }

  const channelData = windowData.channels[0];
  const unit = channelData?.unit ?? channelUnits[selectedIndex] ?? null;
  const channelName =
    channelData?.channel_name ?? channelNames[selectedIndex] ?? `Channel ${selectedIndex}`;

  const maxTime = durationSeconds ?? endSeconds;
  const step = Math.max(0.1, maxTime / 100);

  return (
    <div className='space-y-4'>
      <div className='flex flex-col gap-4 md:flex-row md:items-center md:justify-between'>
        <div className='flex flex-col gap-2 md:flex-row md:items-center'>
          <ChannelSelector
            channels={channels}
            value={selectedIndex}
            onChange={handleChannelChange}
            label='Channel'
          />
          <div className='flex items-center gap-2'>
            <label className='text-base-content/60 text-sm'>Window (s)</label>
            <input
              type='number'
              min={0}
              max={maxTime - 0.1}
              step={step}
              value={startSeconds}
              onChange={(e) => handleTimeRangeChange(Number(e.target.value), endSeconds)}
              className='input input-sm w-24'
            />
            <span className='text-base-content/60'>to</span>
            <input
              type='number'
              min={0.1}
              max={maxTime}
              step={step}
              value={endSeconds}
              onChange={(e) => handleTimeRangeChange(startSeconds, Number(e.target.value))}
              className='input input-sm w-24'
            />
            <span className='text-base-content/60 text-sm'>
              Duration: {(endSeconds - startSeconds).toFixed(1)}s
            </span>
          </div>
        </div>
        <div className='flex items-center gap-2'>
          <button
            onClick={prevChannel}
            disabled={totalChannels <= 1}
            className='btn btn-sm btn-ghost'
            aria-label='Previous channel'>
            ← Prev
          </button>
          <button
            onClick={nextChannel}
            disabled={totalChannels <= 1}
            className='btn btn-sm btn-ghost'
            aria-label='Next channel'>
            Next →
          </button>
        </div>
      </div>

      {channelData && (
        <TimeSeriesChart
          channelName={channelName}
          unit={unit}
          data={channelData.samples.map((s) => s ?? 0)}
          samplingFrequencyHz={samplingFrequencyHz}
        />
      )}

      {windowData.downsampled && (
        <p className='text-base-content/60 text-sm'>
          Display is downsampled by {windowData.downsample_factor}x ({windowData.points_per_channel}{' '}
          points per channel of {windowData.sample_end - windowData.sample_start} samples).
        </p>
      )}

      {channelData && (
        <div className='rounded-box border-base-300 border p-4 text-sm'>
          <h4 className='mb-2 font-medium'>Channel data summary</h4>
          <div className='grid gap-2 md:grid-cols-4'>
            <div>
              <span className='text-base-content/60'>Samples:</span>{' '}
              <span className='font-mono'>{channelData.samples.length.toLocaleString()}</span>
            </div>
            <div>
              <span className='text-base-content/60'>Valid:</span>{' '}
              <span className='font-mono'>
                {channelData.samples.filter((s) => s !== null).length.toLocaleString()}
              </span>
            </div>
            <div>
              <span className='text-base-content/60'>Time range:</span>{' '}
              <span className='font-mono'>
                {windowData.start_seconds.toFixed(2)}s – {windowData.end_seconds.toFixed(2)}s
              </span>
            </div>
            <div>
              <span className='text-base-content/60'>Sampling:</span>{' '}
              <span className='font-mono'>{formatFrequency(samplingFrequencyHz)}</span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

function formatFrequency(hz: number): string {
  if (hz >= 1000) {
    return `${(hz / 1000).toFixed(1)} kHz`;
  }
  return `${hz.toFixed(1)} Hz`;
}
