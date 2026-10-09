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

import { PageHeader } from '@/components/molecules/page-header';

describe('PageHeader', () => {
  it('renders the title and optional description', () => {
    render(<PageHeader title='Studies' description='All studies' />);
    expect(screen.getByRole('heading', { level: 1, name: 'Studies' })).toBeInTheDocument();
    expect(screen.getByText('All studies')).toBeInTheDocument();
  });

  it('renders only the title when no optional props are given', () => {
    render(<PageHeader title='Studies' />);
    expect(screen.getByRole('heading', { level: 1, name: 'Studies' })).toBeInTheDocument();
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
    expect(screen.queryByRole('navigation')).not.toBeInTheDocument();
  });

  it('renders optional actions', () => {
    render(<PageHeader title='Studies' actions={<button>New study</button>} />);
    expect(screen.getByRole('button', { name: 'New study' })).toBeInTheDocument();
  });

  it('renders optional breadcrumbs', () => {
    render(
      <PageHeader
        title='Study A'
        breadcrumbs={[{ label: 'Studies', href: '/studies' }, { label: 'Details' }]}
      />,
    );
    expect(screen.getByRole('navigation', { name: 'Breadcrumb' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Studies' })).toHaveAttribute('href', '/studies');
    expect(screen.getByText('Details')).toBeInTheDocument();
  });

  it('renders optional children', () => {
    render(
      <PageHeader title='Studies'>
        <p>Extra content</p>
      </PageHeader>,
    );
    expect(screen.getByText('Extra content')).toBeInTheDocument();
  });
});
