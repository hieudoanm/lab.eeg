import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { AnchorHTMLAttributes, ReactNode } from 'react';

const nav = vi.hoisted(() => ({ pathname: '/app/dashboard' as string | null }));

vi.mock('next/navigation', () => ({
  usePathname: () => nav.pathname,
}));

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

import { AppShell } from '@/components/organisms/app-shell';

describe('AppShell', () => {
  it('renders the top bar and children inside the main landmark', () => {
    nav.pathname = '/app/dashboard';
    render(
      <AppShell topBar={<div>Top bar content</div>}>
        <p>Page body</p>
      </AppShell>,
    );

    expect(screen.getByText('Top bar content')).toBeInTheDocument();
    expect(screen.getByText('Page body')).toBeInTheDocument();
    expect(screen.getByRole('main')).toHaveAttribute('id', 'main-content');
    expect(screen.getByRole('link', { name: 'Skip to main content' })).toBeInTheDocument();
  });

  it('highlights the navigation link that matches the current path', () => {
    nav.pathname = '/app/studies';
    render(
      <AppShell topBar={null}>
        <p>Body</p>
      </AppShell>,
    );

    expect(screen.getByRole('link', { name: /Studies/ })).toHaveAttribute('aria-current', 'page');
    expect(screen.getByRole('link', { name: /Dashboard/ })).not.toHaveAttribute('aria-current');
  });

  it('highlights a parent navigation link for nested routes', () => {
    nav.pathname = '/app/studies/study-1/datasets/dataset-1';
    render(
      <AppShell topBar={null}>
        <p>Body</p>
      </AppShell>,
    );

    expect(screen.getByRole('link', { name: /Studies/ })).toHaveAttribute('aria-current', 'page');
  });

  it('renders no active link when the path is unknown', () => {
    nav.pathname = null;
    render(
      <AppShell topBar={null}>
        <p>Body</p>
      </AppShell>,
    );

    expect(screen.getByRole('link', { name: /Dashboard/ })).not.toHaveAttribute('aria-current');
    expect(screen.getByRole('link', { name: /Studies/ })).not.toHaveAttribute('aria-current');
  });
});
