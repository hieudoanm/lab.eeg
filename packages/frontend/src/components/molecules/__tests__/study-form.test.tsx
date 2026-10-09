import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/trpc/react', async () => {
  const { buildTrpcMock } = await import('@/test/trpc-mock');
  return buildTrpcMock();
});

const { push, refresh } = vi.hoisted(() => ({ push: vi.fn(), refresh: vi.fn() }));

vi.mock('next/navigation', () => ({ useRouter: () => ({ push, refresh }) }));

import { StudyForm } from '@/components/molecules/study-form';
import { mutationController, resetTrpcMocks } from '@/test/trpc-mock';

const setName = (value: string) =>
  fireEvent.change(screen.getByLabelText(/Name/), { target: { value } });

const submitForm = (name: string) => {
  const form = screen.getByRole('button', { name }).closest('form');
  if (!form) {
    throw new Error('form not found');
  }
  fireEvent.submit(form);
};

beforeEach(() => {
  resetTrpcMocks();
  push.mockReset();
  refresh.mockReset();
});

describe('StudyForm', () => {
  it('renders the create fields', () => {
    render(<StudyForm />);

    expect(screen.getByLabelText(/Name/)).toBeInTheDocument();
    expect(screen.getByLabelText(/Description/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Create study' })).toBeInTheDocument();
  });

  it('seeds the fields and submit label when editing', () => {
    render(<StudyForm study={{ id: 's1', name: 'Pilot', description: 'First run' }} />);

    expect(screen.getByLabelText(/Name/)).toHaveValue('Pilot');
    expect(screen.getByLabelText(/Description/)).toHaveValue('First run');
    expect(screen.getByRole('button', { name: 'Save changes' })).toBeInTheDocument();
  });

  it('requires a study name before submitting', async () => {
    render(<StudyForm />);

    submitForm('Create study');

    expect(await screen.findByRole('alert')).toHaveTextContent('Enter a study name.');
    expect(mutationController('studies.create').calls).toHaveLength(0);
  });

  it('creates a study and navigates to it', async () => {
    mutationController('studies.create').resolveWith({ id: 'study-9' });
    render(<StudyForm />);

    setName('  Alpha  ');
    submitForm('Create study');

    await waitFor(() =>
      expect(mutationController('studies.create').calls).toEqual([
        { name: 'Alpha', description: null },
      ]),
    );
    await waitFor(() => expect(push).toHaveBeenCalledWith('/app/studies/study-9'));
    expect(refresh).toHaveBeenCalled();
  });

  it('updates a study and shows a success message', async () => {
    mutationController('studies.update').resolveWith({ id: 's1' });
    render(<StudyForm study={{ id: 's1', name: 'Pilot', description: 'First run' }} />);

    fireEvent.change(screen.getByLabelText(/Description/), { target: { value: 'Updated' } });
    submitForm('Save changes');

    await waitFor(() =>
      expect(mutationController('studies.update').calls[0]).toEqual({
        studyId: 's1',
        name: 'Pilot',
        description: 'Updated',
      }),
    );
    expect(await screen.findByRole('status')).toHaveTextContent('Study updated.');
    expect(refresh).toHaveBeenCalled();
  });

  it('surfaces a mutation failure', async () => {
    mutationController('studies.create').rejectWith(new Error('Boom'));
    render(<StudyForm />);

    setName('Alpha');
    submitForm('Create study');

    expect(await screen.findByRole('alert')).toHaveTextContent('Boom');
  });

  it('shows a creating pending state', async () => {
    mutationController('studies.create').resolveWith(new Promise(() => undefined));
    render(<StudyForm />);

    setName('Alpha');
    submitForm('Create study');

    expect(await screen.findByRole('button', { name: 'Creating…' })).toBeDisabled();
  });

  it('shows a saving pending state while editing', async () => {
    mutationController('studies.update').resolveWith(new Promise(() => undefined));
    render(<StudyForm study={{ id: 's1', name: 'Pilot', description: null }} />);

    submitForm('Save changes');

    expect(await screen.findByRole('button', { name: 'Saving…' })).toBeDisabled();
  });
});
