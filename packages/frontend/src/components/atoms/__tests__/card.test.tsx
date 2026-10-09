import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Card } from '../../atoms/card';

describe('Card', () => {
  it('renders title description and children', () => {
    render(
      <Card title='Card Title' description='Card description'>
        <p>Child content</p>
      </Card>,
    );
    expect(screen.getByRole('heading', { name: 'Card Title' })).toBeInTheDocument();
    expect(screen.getByText('Card description')).toBeInTheDocument();
    expect(screen.getByText('Child content')).toBeInTheDocument();
  });

  it('renders without description', () => {
    render(<Card title='Title'>Content</Card>);
    expect(screen.getByRole('heading', { name: 'Title' })).toBeInTheDocument();
    expect(screen.getByText('Content')).toBeInTheDocument();
  });

  it('renders an optional action alongside the title', () => {
    render(
      <Card title='Title' actions={<button>Edit</button>}>
        Content
      </Card>,
    );
    expect(screen.getByRole('button', { name: 'Edit' })).toBeInTheDocument();
  });

  it('renders children only when no title is provided', () => {
    render(<Card>Just children</Card>);
    expect(screen.queryByRole('heading')).not.toBeInTheDocument();
    expect(screen.getByText('Just children')).toBeInTheDocument();
  });

  it('merges a custom className onto the section', () => {
    render(<Card className='custom-class'>Content</Card>);
    const section = screen.getByText('Content').closest('section');
    expect(section).toHaveClass('custom-class');
    expect(section?.className).toContain('card');
    expect(section?.className).toContain('bg-base-100');
  });
});
