import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { MetadataGrid } from '../../atoms/metadata-grid';

describe('MetadataGrid', () => {
  it('renders label and value pairs', () => {
    render(
      <MetadataGrid
        items={[
          { label: 'Study', value: 'Study A' },
          { label: 'Channels', value: '64' },
        ]}
      />,
    );
    expect(screen.getByText('Study')).toBeInTheDocument();
    expect(screen.getByText('Study A')).toBeInTheDocument();
    expect(screen.getByText('Channels')).toBeInTheDocument();
    expect(screen.getByText('64')).toBeInTheDocument();
  });

  it('renders nullable values', () => {
    render(<MetadataGrid items={[{ label: 'Owner', value: null }]} />);
    expect(screen.getByText('Owner')).toBeInTheDocument();
  });

  it('applies mono styling to flagged values', () => {
    render(<MetadataGrid items={[{ label: 'Checksum', value: 'abc123', mono: true }]} />);
    expect(screen.getByText('abc123')).toHaveClass('font-mono');
  });

  it('supports each column count', () => {
    const { container, rerender } = render(<MetadataGrid items={[]} columns={1} />);
    const list = () => container.querySelector('dl');
    expect(list()).toHaveClass('grid-cols-1');
    rerender(<MetadataGrid items={[]} columns={2} />);
    expect(list()?.className).toContain('sm:grid-cols-2');
    rerender(<MetadataGrid items={[]} columns={3} />);
    expect(list()?.className).toContain('lg:grid-cols-3');
  });
});
