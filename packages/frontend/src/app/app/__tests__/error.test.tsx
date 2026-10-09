import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import AppError from '@/app/app/error';

describe('AppError', () => {
  it('renders the error message and retries', async () => {
    const reset = vi.fn();
    const user = userEvent.setup();
    render(
      <AppError error={Object.assign(new Error('Boom'), { digest: 'digest-1' })} reset={reset} />,
    );

    expect(screen.getByRole('alert')).toHaveTextContent('The workspace could not be loaded');
    expect(screen.getByText('Boom')).toBeInTheDocument();
    expect(screen.getByText('Request ID: digest-1')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Try again' }));
    expect(reset).toHaveBeenCalledTimes(1);
  });

  it('falls back to a safe message when the error has none', () => {
    render(<AppError error={new Error('')} reset={vi.fn()} />);

    expect(screen.getByText('An unexpected error occurred.')).toBeInTheDocument();
  });
});
