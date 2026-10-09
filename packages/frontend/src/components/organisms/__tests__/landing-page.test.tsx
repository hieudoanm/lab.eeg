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

import { LandingPage } from '@/components/organisms/landing-page';
import { ThemeProvider } from '@/components/atoms/theme-provider';

const renderLanding = () =>
  render(
    <ThemeProvider>
      <LandingPage />
    </ThemeProvider>,
  );

describe('LandingPage', () => {
  it('presents the primary value proposition as the level-one heading', () => {
    renderLanding();

    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(
      'Organise studies, explore EEG signals, and run reproducible analyses in the cloud.',
    );
  });

  it('renders the in-page section navigation', () => {
    renderLanding();

    const nav = screen.getByRole('navigation', { name: 'Landing' });
    expect(nav).toBeInTheDocument();
    for (const label of ['Explore', 'Analyse', 'Organise', 'Docs']) {
      expect(screen.getByRole('link', { name: label })).toBeInTheDocument();
    }
  });

  it('routes every sign-in call to action to the sign-in page', () => {
    renderLanding();

    const signInLinks = screen.getAllByRole('link', { name: 'Sign in' });
    expect(signInLinks.length).toBeGreaterThanOrEqual(1);
    for (const link of signInLinks) {
      expect(link).toHaveAttribute('href', '/login');
    }
  });

  it('describes the three capabilities', () => {
    renderLanding();

    expect(screen.getByRole('heading', { name: 'Explore' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Analyse' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Organise' })).toBeInTheDocument();
  });

  it('exposes an accessible textual alternative for the signal preview', () => {
    renderLanding();

    expect(
      screen.getByRole('img', {
        name: /Multichannel EEG signal preview with channel selection and time navigation/,
      }),
    ).toBeInTheDocument();
  });

  it('summarises the public sample recording with its provenance', () => {
    renderLanding();

    expect(screen.getByRole('heading', { name: 'Public sample recording' })).toBeInTheDocument();
    expect(screen.getByText('sha256:8f2d3b9c5e…f315d9e7c22f6e04')).toBeInTheDocument();
    expect(screen.getByText(/Not for clinical use/)).toBeInTheDocument();
  });

  it('renders the footer link groups and disclaimer', () => {
    renderLanding();

    expect(screen.getByText('Product')).toBeInTheDocument();
    expect(screen.getByText('Documentation')).toBeInTheDocument();
    expect(screen.getByText('Legal')).toBeInTheDocument();
    expect(
      screen.getByText('© 2026 EEG Lab · Research and educational use only. Not a medical device.'),
    ).toBeInTheDocument();
  });
});
