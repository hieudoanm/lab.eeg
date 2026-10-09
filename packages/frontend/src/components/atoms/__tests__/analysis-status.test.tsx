import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import {
  AnalysisStatusBadge,
  AnalysisStatusDetail,
  analysisStatusCanCancel,
  analysisStatusIsTerminal,
} from '../analysis-status';
import type { AnalysisJobStatus } from '@/lib/api';

const STATUS_LABELS: Array<[AnalysisJobStatus, string]> = [
  ['queued', 'Queued'],
  ['running', 'Running'],
  ['succeeded', 'Succeeded'],
  ['failed', 'Failed'],
  ['cancelled', 'Cancelled'],
];

describe('AnalysisStatusBadge', () => {
  it.each(STATUS_LABELS)('renders the %s status as %s', (status, label) => {
    render(<AnalysisStatusBadge status={status} />);
    expect(screen.getByText(label)).toBeInTheDocument();
  });
});

describe('AnalysisStatusDetail', () => {
  it.each(STATUS_LABELS)('renders the %s status badge', (status, label) => {
    render(<AnalysisStatusDetail status={status} attemptCount={1} maxAttempts={3} error={null} />);
    expect(screen.getByText(label)).toBeInTheDocument();
  });

  it('shows the attempt count while running', () => {
    render(<AnalysisStatusDetail status='running' attemptCount={2} maxAttempts={5} error={null} />);
    expect(screen.getByText('Attempt 2 of 5')).toBeInTheDocument();
  });

  it('shows a max-attempts notice when a failed job has exhausted retries', () => {
    render(<AnalysisStatusDetail status='failed' attemptCount={5} maxAttempts={5} error={null} />);
    expect(screen.getByText('Max attempts reached')).toBeInTheDocument();
  });

  it('does not show a max-attempts notice before the limit is reached', () => {
    render(<AnalysisStatusDetail status='failed' attemptCount={4} maxAttempts={5} error={null} />);
    expect(screen.queryByText('Max attempts reached')).not.toBeInTheDocument();
  });

  it('renders error code and message when present', () => {
    render(
      <AnalysisStatusDetail
        status='failed'
        attemptCount={5}
        maxAttempts={5}
        error={{ code: 'ANALYSIS_FAILED', message: 'boom' }}
      />,
    );
    expect(screen.getByText('ANALYSIS_FAILED')).toBeInTheDocument();
    expect(
      screen.getByText((_, element) => element?.textContent === 'ANALYSIS_FAILED: boom'),
    ).toBeInTheDocument();
  });
});

describe('analysisStatusIsTerminal', () => {
  it.each<[AnalysisJobStatus, boolean]>([
    ['queued', false],
    ['running', false],
    ['succeeded', true],
    ['failed', true],
    ['cancelled', true],
  ])('returns %s => %s', (status, expected) => {
    expect(analysisStatusIsTerminal(status)).toBe(expected);
  });
});

describe('analysisStatusCanCancel', () => {
  it.each<[AnalysisJobStatus, boolean]>([
    ['queued', true],
    ['running', false],
    ['succeeded', false],
    ['failed', false],
    ['cancelled', false],
  ])('returns %s => %s', (status, expected) => {
    expect(analysisStatusCanCancel(status)).toBe(expected);
  });
});
