import { act, fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { ToastProvider, useToast, type ToastInput } from '@/components/atoms/toast';

const Notifier = ({
  input = { title: 'Analysis queued', tone: 'success' },
}: {
  input?: ToastInput;
}) => {
  const { showToast } = useToast();
  return (
    <button type='button' onClick={() => showToast(input)}>
      Notify
    </button>
  );
};

const OutsideProvider = () => {
  useToast();
  return null;
};

afterEach(() => {
  vi.useRealTimers();
});

describe('ToastProvider', () => {
  it('shows a notification when requested', async () => {
    const user = userEvent.setup();
    render(
      <ToastProvider>
        <Notifier />
      </ToastProvider>,
    );

    await user.click(screen.getByRole('button', { name: 'Notify' }));

    expect(await screen.findByText('Analysis queued')).toBeInTheDocument();
  });

  it('renders the optional description', async () => {
    render(
      <ToastProvider>
        <Notifier input={{ title: 'Recording ready', description: 'Validated successfully.' }} />
      </ToastProvider>,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Notify' }));

    expect(await screen.findByText('Recording ready')).toBeInTheDocument();
    expect(screen.getByText('Validated successfully.')).toBeInTheDocument();
  });

  it.each(['info', 'success', 'danger'] as const)('renders the %s tone styling', (tone) => {
    render(
      <ToastProvider>
        <Notifier input={{ title: `A ${tone} message`, tone }} />
      </ToastProvider>,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Notify' }));

    const alertTone = tone === 'danger' ? 'error' : tone;
    expect(screen.getByRole('status')).toHaveClass(`alert-${alertTone}`);
  });

  it('dismisses a notification when the dismiss button is pressed', async () => {
    const user = userEvent.setup();
    render(
      <ToastProvider>
        <Notifier />
      </ToastProvider>,
    );

    await user.click(screen.getByRole('button', { name: 'Notify' }));
    expect(await screen.findByText('Analysis queued')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Dismiss notification' }));

    expect(screen.queryByText('Analysis queued')).not.toBeInTheDocument();
  });

  it('auto-dismisses a notification after six seconds', () => {
    vi.useFakeTimers();
    render(
      <ToastProvider>
        <Notifier />
      </ToastProvider>,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Notify' }));
    expect(screen.getByText('Analysis queued')).toBeInTheDocument();

    act(() => {
      vi.advanceTimersByTime(6000);
    });

    expect(screen.queryByText('Analysis queued')).not.toBeInTheDocument();
  });

  it('generates an id when crypto.randomUUID is unavailable', () => {
    const original = globalThis.crypto.randomUUID;
    Object.defineProperty(globalThis.crypto, 'randomUUID', {
      value: undefined,
      configurable: true,
    });
    try {
      render(
        <ToastProvider>
          <Notifier />
        </ToastProvider>,
      );

      fireEvent.click(screen.getByRole('button', { name: 'Notify' }));

      expect(screen.getByText('Analysis queued')).toBeInTheDocument();
    } finally {
      Object.defineProperty(globalThis.crypto, 'randomUUID', {
        value: original,
        configurable: true,
      });
    }
  });

  it('keeps other notifications when one is dismissed', async () => {
    const user = userEvent.setup();
    render(
      <ToastProvider>
        <Notifier />
        <Notifier input={{ title: 'Second message' }} />
      </ToastProvider>,
    );

    const [first, second] = screen.getAllByRole('button', { name: 'Notify' });
    await user.click(first);
    await user.click(second);

    expect(await screen.findByText('Second message')).toBeInTheDocument();

    const dismissButtons = screen.getAllByRole('button', { name: 'Dismiss notification' });
    await user.click(dismissButtons[0]);

    expect(screen.queryByText('Analysis queued')).not.toBeInTheDocument();
    expect(screen.getByText('Second message')).toBeInTheDocument();
  });
});

describe('useToast', () => {
  it('throws when used outside a provider', () => {
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => undefined);

    expect(() => render(<OutsideProvider />)).toThrow(
      'useToast must be used within a ToastProvider',
    );

    consoleError.mockRestore();
  });
});
