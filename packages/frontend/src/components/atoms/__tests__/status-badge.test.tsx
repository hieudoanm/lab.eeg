import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { StatusBadge, type StatusTone } from '../../atoms/status-badge';

describe('StatusBadge', () => {
  const tones: StatusTone[] = ['neutral', 'info', 'success', 'warning', 'danger'];

  it.each(tones)('renders the label for the %s tone', (tone) => {
    render(<StatusBadge tone={tone} label={`status-${tone}`} />);
    expect(screen.getByText(`status-${tone}`)).toBeInTheDocument();
  });

  it('applies tone classes and merges a custom className', () => {
    render(<StatusBadge tone='danger' label='Failed' className='custom-badge' />);
    const badge = screen.getByText('Failed');
    expect(badge).toHaveClass('custom-badge');
    expect(badge.className).toContain('badge-error');
  });

  it('hides the decorative dot from assistive technology', () => {
    const { container } = render(<StatusBadge tone='info' label='Running' />);
    expect(container.querySelector("span[aria-hidden='true']")).toBeInTheDocument();
  });
});
