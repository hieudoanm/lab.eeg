import type { AnchorHTMLAttributes, ReactNode } from 'react';

import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

vi.mock('next/link', () => ({
  default: ({
    href,
    children,
    ...rest
  }: { href: string; children: ReactNode } & AnchorHTMLAttributes<HTMLAnchorElement>) => (
    <a href={href} {...rest}>
      {children}
    </a>
  ),
}));

import { Breadcrumbs } from '@/components/atoms/breadcrumbs';

describe('Breadcrumbs', () => {
  it('renders links for every item except the last', () => {
    render(
      <Breadcrumbs
        items={[
          { label: 'Studies', href: '/studies' },
          { label: 'Study A', href: '/studies/a' },
          { label: 'Dataset' },
        ]}
      />,
    );
    expect(screen.getByRole('link', { name: 'Studies' })).toHaveAttribute('href', '/studies');
    expect(screen.getByRole('link', { name: 'Study A' })).toHaveAttribute('href', '/studies/a');
    expect(screen.queryByRole('link', { name: 'Dataset' })).not.toBeInTheDocument();
  });

  it('marks the last item as the current page', () => {
    render(<Breadcrumbs items={[{ label: 'Studies', href: '/studies' }, { label: 'Study A' }]} />);
    expect(screen.getByText('Study A')).toHaveAttribute('aria-current', 'page');
  });

  it('renders a non-linked item without href as plain text', () => {
    render(<Breadcrumbs items={[{ label: 'Home' }]} />);
    expect(screen.queryByRole('link')).not.toBeInTheDocument();
    expect(screen.getByText('Home')).toHaveAttribute('aria-current', 'page');
  });

  it('renders a non-last item without href as plain text without aria-current', () => {
    render(<Breadcrumbs items={[{ label: 'Home' }, { label: 'Studies', href: '/studies' }]} />);
    const home = screen.getByText('Home');
    expect(home.tagName).toBe('SPAN');
    expect(home).not.toHaveAttribute('aria-current');
  });

  it('renders nothing for an empty list', () => {
    const { container } = render(<Breadcrumbs items={[]} />);
    expect(container).toBeEmptyDOMElement();
    expect(screen.queryByRole('navigation')).not.toBeInTheDocument();
  });
});
