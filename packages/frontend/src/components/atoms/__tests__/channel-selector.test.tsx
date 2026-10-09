import { act, render, renderHook, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { ChannelSelector, useChannelSelection } from '../../molecules/channel-selector';

const CHANNELS = [
  { index: 0, name: 'Fp1' },
  { index: 1, name: 'Fp2' },
  { index: 3, name: 'Cz' },
];

describe('ChannelSelector', () => {
  it('renders each channel as an option', () => {
    render(<ChannelSelector channels={CHANNELS} value={0} onChange={vi.fn()} />);

    const select = screen.getByLabelText('Channel');
    expect(select).toHaveValue('0');
    expect(screen.getByRole('option', { name: 'Fp1' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Fp2' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Cz' })).toBeInTheDocument();
  });

  it('supports a custom label', () => {
    render(
      <ChannelSelector channels={CHANNELS} value={1} onChange={vi.fn()} label='Primary channel' />,
    );
    expect(screen.getByLabelText('Primary channel')).toBeInTheDocument();
  });

  it('reports selection changes as numbers', async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    render(<ChannelSelector channels={CHANNELS} value={0} onChange={onChange} />);

    await user.selectOptions(screen.getByLabelText('Channel'), '3');

    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange).toHaveBeenCalledWith(3);
  });

  it('disables the control when requested', () => {
    render(<ChannelSelector channels={CHANNELS} value={0} onChange={vi.fn()} disabled />);
    expect(screen.getByLabelText('Channel')).toBeDisabled();
  });
});

describe('useChannelSelection', () => {
  it('initialises to the requested index', () => {
    const { result } = renderHook(() => useChannelSelection(CHANNELS, 1));
    expect(result.current.selectedIndex).toBe(1);
    expect(result.current.selectedChannel).toEqual({ index: 1, name: 'Fp2' });
    expect(result.current.channelCount).toBe(3);
  });

  it('clamps the initial index into range', () => {
    const { result } = renderHook(() => useChannelSelection(CHANNELS, 99));
    expect(result.current.selectedIndex).toBe(2);
  });

  it('clamps explicit selections', () => {
    const { result } = renderHook(() => useChannelSelection(CHANNELS, 0));

    act(() => result.current.setChannel(99));
    expect(result.current.selectedIndex).toBe(2);

    act(() => result.current.setChannel(-5));
    expect(result.current.selectedIndex).toBe(0);
  });

  it('advances and wraps with nextChannel', () => {
    const { result } = renderHook(() => useChannelSelection(CHANNELS, 1));

    act(() => result.current.nextChannel());
    expect(result.current.selectedIndex).toBe(2);

    act(() => result.current.nextChannel());
    expect(result.current.selectedIndex).toBe(0);
  });

  it('retreats and wraps with prevChannel', () => {
    const { result } = renderHook(() => useChannelSelection(CHANNELS, 0));

    act(() => result.current.prevChannel());
    expect(result.current.selectedIndex).toBe(2);

    act(() => result.current.prevChannel());
    expect(result.current.selectedIndex).toBe(1);
  });
});
