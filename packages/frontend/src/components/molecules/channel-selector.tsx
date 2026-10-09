'use client';

import { useState, useCallback } from 'react';
import { SelectField } from '@/components/atoms/select-field';

export interface ChannelOption {
  index: number;
  name: string;
}

export interface ChannelSelectorProps {
  channels: ChannelOption[];
  value: number;
  onChange: (index: number) => void;
  label?: string;
  disabled?: boolean;
}

export const ChannelSelector = ({
  channels,
  value,
  onChange,
  label = 'Channel',
  disabled = false,
}: ChannelSelectorProps) => {
  return (
    <SelectField
      label={label}
      value={String(value)}
      onChange={(e) => onChange(Number(e.target.value))}
      options={channels.map((ch) => ({ value: String(ch.index), label: ch.name }))}
      disabled={disabled}
    />
  );
};

export const useChannelSelection = <T extends { name: string }>(
  channels: T[],
  initialIndex: number = 0,
) => {
  const [selectedIndex, setSelectedIndex] = useState(() =>
    Math.max(0, Math.min(initialIndex, channels.length - 1)),
  );

  const setChannel = useCallback(
    (index: number) => {
      setSelectedIndex(Math.max(0, Math.min(index, channels.length - 1)));
    },
    [channels.length],
  );

  const nextChannel = useCallback(() => {
    setSelectedIndex((prev) => (prev + 1) % channels.length);
  }, [channels.length]);

  const prevChannel = useCallback(() => {
    setSelectedIndex((prev) => (prev - 1 + channels.length) % channels.length);
  }, [channels.length]);

  return {
    selectedIndex,
    selectedChannel: channels[selectedIndex],
    setChannel,
    nextChannel,
    prevChannel,
    channelCount: channels.length,
  };
};
