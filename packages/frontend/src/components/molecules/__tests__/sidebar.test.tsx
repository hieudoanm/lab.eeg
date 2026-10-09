import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
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

import { Sidebar } from '@/components/molecules/sidebar';

describe('Sidebar', () => {
  it('renders every primary navigation link', () => {
    nav.pathname = '/app/dashboard';
    render(<Sidebar />);

    const navigation = screen.getByRole('navigation', { name: 'Primary' });
    expect(navigation).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /Dashboard/ })).toHaveAttribute(
      'href',
      '/app/dashboard',
    );
    expect(screen.getByRole('link', { name: /Studies/ })).toHaveAttribute('href', '/app/studies');
    expect(screen.getByRole('link', { name: /Analyses/ })).toHaveAttribute('href', '/app/analyses');
    expect(screen.getByRole('link', { name: /Organisation/ })).toHaveAttribute(
      'href',
      '/app/settings/organisation',
    );
    expect(screen.getByRole('link', { name: /EEG Lab/ })).toHaveAttribute('href', '/app/dashboard');
  });

  it('marks the current section as the active page', () => {
    nav.pathname = '/app/analyses';
    render(<Sidebar />);

    expect(screen.getByRole('link', { name: /Analyses/ })).toHaveAttribute('aria-current', 'page');
  });

  it('marks parent sections active for nested routes', () => {
    nav.pathname = '/app/settings/organisation/members';
    render(<Sidebar />);

    expect(screen.getByRole('link', { name: /Organisation/ })).toHaveAttribute(
      'aria-current',
      'page',
    );
  });

  it('opens and closes the mobile menu', async () => {
    nav.pathname = '/app/dashboard';
    const user = userEvent.setup();
    render(<Sidebar />);

    const toggle = screen.getByRole('button', { name: /Menu/ });
    expect(toggle).toHaveAttribute('aria-expanded', 'false');

    await user.click(toggle);
    expect(toggle).toHaveAttribute('aria-expanded', 'true');

    await user.click(screen.getByRole('button', { name: 'Close navigation' }));
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
  });

  it('closes the mobile menu when a link is followed', async () => {
    nav.pathname = '/app/dashboard';
    const user = userEvent.setup();
    render(<Sidebar />);

    await user.click(screen.getByRole('button', { name: /Menu/ }));
    await user.click(screen.getByRole('link', { name: /Studies/ }));

    expect(screen.getByRole('button', { name: /Menu/ })).toHaveAttribute('aria-expanded', 'false');
  });

  it('closes the mobile menu when the backdrop is clicked', async () => {
    nav.pathname = '/app/dashboard';
    const { container } = render(<Sidebar />);

    fireEvent.click(screen.getByRole('button', { name: /Menu/ }));
    const backdrop = container.querySelector('div.fixed.inset-0');
    if (!backdrop) {
      throw new Error('Backdrop not rendered');
    }
    fireEvent.click(backdrop);

    expect(screen.getByRole('button', { name: /Menu/ })).toHaveAttribute('aria-expanded', 'false');
  });
});
