import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { EmptyState, ErrorState, LoadingState, NoResultsState } from '../states';

describe('states', () => {
  it('renders loading state', () => {
    render(<LoadingState label='Loading data' />);
    expect(screen.getByRole('status')).toHaveTextContent('Loading data');
  });

  it('renders the default loading label and merges className', () => {
    render(<LoadingState className='custom-loading' />);
    const status = screen.getByRole('status');
    expect(status).toHaveTextContent('Loading…');
    expect(status).toHaveClass('custom-loading');
  });

  it('renders empty state', () => {
    render(
      <EmptyState title='No items' description='No items found' action={<button>New</button>} />,
    );
    expect(screen.getByText('No items')).toBeInTheDocument();
    expect(screen.getByText('No items found')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'New' })).toBeInTheDocument();
  });

  it('renders empty state without an action', () => {
    render(<EmptyState title='No items' description='No items found' />);
    expect(screen.getByRole('heading', { name: 'No items' })).toBeInTheDocument();
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('renders error state', () => {
    render(<ErrorState message='Something went wrong' requestId='req-123' />);
    expect(screen.getByRole('alert')).toHaveTextContent('Something went wrong');
    expect(screen.getByText('Request ID: req-123')).toBeInTheDocument();
  });

  it('renders the default error title without a request id or action', () => {
    render(<ErrorState message='Boom' />);
    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent('Something went wrong');
    expect(alert).toHaveTextContent('Boom');
    expect(screen.queryByText(/Request ID/)).not.toBeInTheDocument();
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('renders an error action when provided', () => {
    render(<ErrorState message='Boom' action={<button>Retry</button>} />);
    expect(screen.getByRole('button', { name: 'Retry' })).toBeInTheDocument();
  });

  it('renders no results state', () => {
    render(<NoResultsState description='Nothing matches' />);
    expect(screen.getByRole('status')).toHaveTextContent('Nothing matches');
  });

  it('renders the default no-results description', () => {
    render(<NoResultsState />);
    expect(screen.getByRole('status')).toHaveTextContent('No resources match the current filters.');
  });
});
