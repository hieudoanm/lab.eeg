import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { SelectField } from '@/components/atoms/select-field';
import { StatusBadge } from '@/components/atoms/status-badge';
import { TextareaField } from '@/components/atoms/textarea-field';
import { TextField } from '@/components/atoms/text-field';

describe('StatusBadge', () => {
  it('always exposes a text label alongside colour', () => {
    render(<StatusBadge tone='success' label='Ready' />);
    expect(screen.getByText('Ready')).toBeInTheDocument();
  });
});

describe('TextField', () => {
  it('links the label, hint, and error to the input', () => {
    render(
      <TextField
        name='name'
        label='Study name'
        hint='Up to 200 characters.'
        error='Study name must not be blank.'
      />,
    );
    const input = screen.getByLabelText(/Study name/);
    expect(input).toHaveAttribute('aria-invalid', 'true');
    expect(screen.getByRole('alert')).toHaveTextContent('Study name must not be blank.');
    expect(input).toHaveAccessibleDescription(/Up to 200 characters/);
  });

  it('derives the id from the name and renders a placeholder', () => {
    render(<TextField name='study-name' label='Study name' placeholder='My study' />);
    const input = screen.getByLabelText('Study name');
    expect(input).toHaveAttribute('id', 'study-name');
    expect(input).toHaveAttribute('placeholder', 'My study');
  });

  it('marks required fields with a visible marker', () => {
    render(<TextField label='Study name' required />);
    const input = screen.getByLabelText(/Study name/);
    expect(input).toBeRequired();
    expect(screen.getByText('*')).toHaveAttribute('aria-hidden', 'true');
  });

  it('renders a default value', () => {
    render(<TextField label='Study name' defaultValue='Existing' />);
    expect(screen.getByLabelText('Study name')).toHaveValue('Existing');
  });

  it('renders disabled inputs', () => {
    render(<TextField label='Study name' disabled />);
    expect(screen.getByLabelText('Study name')).toBeDisabled();
  });

  it('renders no alert and no aria-invalid when valid', () => {
    render(<TextField label='Study name' />);
    const input = screen.getByLabelText('Study name');
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    expect(input).not.toHaveAttribute('aria-invalid');
  });
});

describe('TextareaField', () => {
  it('associates the label and renders a default value', () => {
    render(<TextareaField name='notes' label='Notes' defaultValue='Some notes' />);
    const textarea = screen.getByLabelText('Notes');
    expect(textarea.tagName).toBe('TEXTAREA');
    expect(textarea).toHaveValue('Some notes');
  });

  it('shows an error with aria-invalid and role alert', () => {
    render(<TextareaField label='Notes' hint='Optional' error='Too long' />);
    const textarea = screen.getByLabelText('Notes');
    expect(textarea).toHaveAttribute('aria-invalid', 'true');
    expect(screen.getByRole('alert')).toHaveTextContent('Too long');
    expect(textarea).toHaveAccessibleDescription(/Optional/);
  });

  it('supports placeholder and disabled', () => {
    render(<TextareaField label='Notes' placeholder='Write notes' disabled />);
    const textarea = screen.getByLabelText('Notes');
    expect(textarea).toHaveAttribute('placeholder', 'Write notes');
    expect(textarea).toBeDisabled();
  });

  it('marks required textareas', () => {
    render(<TextareaField label='Notes' required />);
    expect(screen.getByLabelText(/Notes/)).toBeRequired();
  });
});

describe('SelectField', () => {
  const options = [
    { value: '', label: 'Select a study' },
    { value: 's1', label: 'Study one' },
    { value: 's2', label: 'Study two' },
  ];

  it('renders the label and all options including the empty option', () => {
    render(<SelectField label='Study' options={options} defaultValue='' />);
    expect(screen.getByLabelText('Study')).toBeInTheDocument();
    expect(screen.getAllByRole('option')).toHaveLength(3);
    expect(
      (screen.getByRole('option', { name: 'Select a study' }) as HTMLOptionElement).value,
    ).toBe('');
    expect((screen.getByRole('option', { name: 'Study one' }) as HTMLOptionElement).value).toBe(
      's1',
    );
    expect((screen.getByRole('option', { name: 'Study two' }) as HTMLOptionElement).value).toBe(
      's2',
    );
  });

  it('reflects the selected value', () => {
    render(<SelectField label='Study' options={options} defaultValue='s2' />);
    expect(screen.getByLabelText('Study')).toHaveValue('s2');
  });

  it('renders a hint linked to the select', () => {
    render(<SelectField label='Study' options={options} hint='Pick one' />);
    expect(screen.getByLabelText('Study')).toHaveAccessibleDescription('Pick one');
  });

  it('renders a required marker and an error', () => {
    render(<SelectField label='Study' options={options} required error='Choose a study' />);
    const select = screen.getByLabelText(/Study/);
    expect(select).toBeRequired();
    expect(select).toHaveAttribute('aria-invalid', 'true');
    expect(screen.getByRole('alert')).toHaveTextContent('Choose a study');
  });

  it('renders disabled selects', () => {
    render(<SelectField label='Study' options={options} disabled />);
    expect(screen.getByLabelText('Study')).toBeDisabled();
  });
});
