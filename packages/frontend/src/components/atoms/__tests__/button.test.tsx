import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { Button, buttonClasses } from '../../atoms/button';

describe('buttonClasses', () => {
  it('computes the base classes and falls back to primary/md defaults', () => {
    const value = buttonClasses();
    expect(value).toContain('btn');
    expect(value).toContain('btn-primary');
    expect(value).toContain('btn-md');
  });

  it('maps every variant to its classes', () => {
    expect(buttonClasses('primary', 'md')).toContain('btn-primary');
    expect(buttonClasses('secondary', 'md')).toContain('btn-outline');
    expect(buttonClasses('danger', 'sm')).toContain('btn-error');
    expect(buttonClasses('ghost', 'md')).toContain('btn-ghost');
  });

  it('maps every size to its classes', () => {
    expect(buttonClasses('primary', 'sm')).toContain('btn-sm');
    expect(buttonClasses('primary', 'md')).toContain('btn-md');
  });
});

describe('Button', () => {
  it('renders its label and defaults to a primary md button', () => {
    render(<Button>Save</Button>);
    const btn = screen.getByRole('button', { name: 'Save' });
    expect(btn).toHaveAttribute('type', 'button');
    expect(btn.className).toContain('btn-primary');
    expect(btn.className).toContain('btn-md');
  });

  it('merges a custom className with the variant classes', () => {
    render(
      <Button variant='danger' size='sm' className='custom-class'>
        Delete
      </Button>,
    );
    const btn = screen.getByRole('button', { name: 'Delete' });
    expect(btn).toHaveClass('custom-class');
    expect(btn.className).toContain('btn-error');
    expect(btn.className).toContain('btn-sm');
  });

  it('calls onClick when pressed', () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Click me</Button>);
    fireEvent.click(screen.getByRole('button', { name: 'Click me' }));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('exposes the disabled state and does not fire onClick', () => {
    const onClick = vi.fn();
    render(
      <Button disabled onClick={onClick}>
        Disabled
      </Button>,
    );
    const btn = screen.getByRole('button', { name: 'Disabled' });
    expect(btn).toBeDisabled();
    expect(btn.className).toContain('disabled:opacity-50');
    fireEvent.click(btn);
    expect(onClick).not.toHaveBeenCalled();
  });

  it('honours an explicit submit type', () => {
    render(<Button type='submit'>Submit</Button>);
    expect(screen.getByRole('button', { name: 'Submit' })).toHaveAttribute('type', 'submit');
  });
});
