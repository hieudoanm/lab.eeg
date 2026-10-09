import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { AnchorHTMLAttributes, ReactNode } from 'react';

vi.mock('next/link', () => ({
  default: ({
    href,
    children,
    ...rest
  }: AnchorHTMLAttributes<HTMLAnchorElement> & { href: string; children: ReactNode }) => (
    <a href={href} {...rest}>
      {children}
    </a>
  ),
}));

import RootPage, { metadata } from '@/app/page';
import { ThemeProvider } from '@/components/atoms/theme-provider';

describe('RootPage', () => {
  it('renders the public landing page', () => {
    render(
      <ThemeProvider>
        <RootPage />
      </ThemeProvider>,
    );

    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(
      'Organise studies, explore EEG signals',
    );
  });

  it('sets the landing page metadata', () => {
    expect(metadata.title).toBe('EEG Lab — Organise, explore, and analyse EEG in the cloud');
  });
});
