import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/trpc/react', async () => {
  const { buildTrpcMock } = await import('@/test/trpc-mock');
  return buildTrpcMock();
});

const { push, refresh } = vi.hoisted(() => ({ push: vi.fn(), refresh: vi.fn() }));

vi.mock('next/navigation', () => ({ useRouter: () => ({ push, refresh }) }));

import { DatasetForm } from '@/components/molecules/dataset-form';
import { mutationController, resetTrpcMocks } from '@/test/trpc-mock';

const setName = (value: string) =>
  fireEvent.change(screen.getByLabelText(/Name/), { target: { value } });

const submitForm = () => {
  const form = screen.getByRole('button', { name: 'Register dataset' }).closest('form');
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

describe('DatasetForm', () => {
  it('renders the registration fields', () => {
    render(<DatasetForm studyId='s1' />);

    expect(screen.getByLabelText(/Name/)).toBeInTheDocument();
    expect(screen.getByLabelText(/Description/)).toBeInTheDocument();
    expect(screen.getByLabelText(/Source/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Register dataset' })).toBeInTheDocument();
  });

  it('reports a missing name without calling the API', () => {
    render(<DatasetForm studyId='s1' />);

    submitForm();

    expect(screen.getByRole('alert')).toHaveTextContent('Enter a dataset name.');
    expect(mutationController('datasets.register').calls).toHaveLength(0);
  });

  it('registers a dataset and navigates to it', async () => {
    mutationController('datasets.register').resolveWith({ id: 'dataset-9' });
    render(<DatasetForm studyId='s1' />);

    setName('Recording 1');
    submitForm();

    await waitFor(() =>
      expect(mutationController('datasets.register').calls[0]).toEqual({
        studyId: 's1',
        name: 'Recording 1',
        description: null,
        sourceDescription: null,
      }),
    );
    expect(push).toHaveBeenCalledWith('/app/studies/s1/datasets/dataset-9');
    expect(refresh).toHaveBeenCalled();
  });

  it('surfaces a registration failure', async () => {
    mutationController('datasets.register').rejectWith(new Error('Boom'));
    render(<DatasetForm studyId='s1' />);

    setName('Recording 1');
    submitForm();

    expect(await screen.findByRole('alert')).toHaveTextContent('Boom');
  });

  it('shows a registering pending state', async () => {
    mutationController('datasets.register').resolveWith(new Promise(() => undefined));
    render(<DatasetForm studyId='s1' />);

    setName('Recording 1');
    submitForm();

    expect(await screen.findByRole('button', { name: 'Registering…' })).toBeDisabled();
  });
});
