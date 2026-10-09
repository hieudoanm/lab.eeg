import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/trpc/react', async () => {
  const { buildTrpcMock } = await import('@/test/trpc-mock');
  return buildTrpcMock();
});

import { ArchiveStudyButton } from '@/components/atoms/archive-study-button';
import { mutationController, resetTrpcMocks } from '@/test/trpc-mock';

beforeEach(() => {
  resetTrpcMocks();
});

describe('ArchiveStudyButton', () => {
  it('requires confirmation before archiving', async () => {
    mutationController('studies.archive').resolveWith(undefined);
    const user = userEvent.setup();
    render(<ArchiveStudyButton studyId='s1' />);

    await user.click(screen.getByRole('button', { name: 'Archive study' }));
    expect(mutationController('studies.archive').calls).toHaveLength(0);

    await user.click(screen.getByRole('button', { name: 'Confirm archive' }));

    await waitFor(() =>
      expect(mutationController('studies.archive').calls[0]).toEqual({ studyId: 's1' }),
    );
  });

  it('can cancel the confirmation', async () => {
    const user = userEvent.setup();
    render(<ArchiveStudyButton studyId='s1' />);

    await user.click(screen.getByRole('button', { name: 'Archive study' }));
    await user.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(screen.getByRole('button', { name: 'Archive study' })).toBeInTheDocument();
    expect(mutationController('studies.archive').calls).toHaveLength(0);
  });

  it('surfaces an archive failure', async () => {
    mutationController('studies.archive').rejectWith(new Error('You cannot archive this study.'));
    const user = userEvent.setup();
    render(<ArchiveStudyButton studyId='s1' />);

    await user.click(screen.getByRole('button', { name: 'Archive study' }));
    await user.click(screen.getByRole('button', { name: 'Confirm archive' }));

    expect(await screen.findByRole('alert')).toHaveTextContent('You cannot archive this study.');
  });

  it('archives without showing an error on success', async () => {
    mutationController('studies.archive').resolveWith(undefined);
    const user = userEvent.setup();
    render(<ArchiveStudyButton studyId='s1' />);

    await user.click(screen.getByRole('button', { name: 'Archive study' }));
    await user.click(screen.getByRole('button', { name: 'Confirm archive' }));

    await waitFor(() => expect(mutationController('studies.archive').calls).toHaveLength(1));
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  it('disables the controls while archiving', async () => {
    let resolveArchive: (value: undefined) => void = () => undefined;
    mutationController('studies.archive').resolveWith(
      new Promise<undefined>((resolve) => {
        resolveArchive = resolve;
      }),
    );
    const user = userEvent.setup();
    render(<ArchiveStudyButton studyId='s1' />);

    await user.click(screen.getByRole('button', { name: 'Archive study' }));
    await user.click(screen.getByRole('button', { name: 'Confirm archive' }));

    const pending = await screen.findByRole('button', { name: 'Archiving…' });
    expect(pending).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeDisabled();

    resolveArchive(undefined);
    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'Confirm archive' })).not.toBeDisabled(),
    );
  });

  it('clears a previous failure on a successful retry', async () => {
    mutationController('studies.archive')
      .rejectWith(new Error('You cannot archive this study.'))
      .resolveWith(undefined);
    const user = userEvent.setup();
    render(<ArchiveStudyButton studyId='s1' />);

    await user.click(screen.getByRole('button', { name: 'Archive study' }));
    await user.click(screen.getByRole('button', { name: 'Confirm archive' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('You cannot archive this study.');

    await user.click(screen.getByRole('button', { name: 'Confirm archive' }));
    await waitFor(() => expect(screen.queryByRole('alert')).not.toBeInTheDocument());
    expect(mutationController('studies.archive').calls).toHaveLength(2);
  });
});
