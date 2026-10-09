import { fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { ThemeProvider } from '../theme-provider';
import { ThemeToggle } from '../theme-toggle';

const stubMatchMedia = (prefersDark: boolean) => {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: vi.fn().mockImplementation((query: string) => ({
      matches: query.includes('dark') ? prefersDark : false,
      media: query,
      onchange: null,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  });
};

const renderToggle = () =>
  render(
    <ThemeProvider>
      <ThemeToggle />
    </ThemeProvider>,
  );

describe('ThemeToggle', () => {
  beforeEach(() => {
    window.localStorage.clear();
    delete document.documentElement.dataset.theme;
    stubMatchMedia(false);
  });

  afterEach(() => {
    delete document.documentElement.dataset.theme;
  });

  it('starts in the light theme when the system prefers light', () => {
    renderToggle();
    expect(document.documentElement.dataset.theme).toBe('lab-light');
    expect(screen.getByRole('button', { name: 'Use light theme' })).toHaveAttribute(
      'aria-pressed',
      'true',
    );
  });

  it('starts in the dark theme when the system prefers dark', () => {
    stubMatchMedia(true);
    renderToggle();
    expect(document.documentElement.dataset.theme).toBe('lab-dark');
    expect(screen.getByRole('button', { name: 'Use dark theme' })).toHaveAttribute(
      'aria-pressed',
      'true',
    );
  });

  it('applies and persists an explicit choice', () => {
    renderToggle();
    fireEvent.click(screen.getByRole('button', { name: 'Use dark theme' }));
    expect(document.documentElement.dataset.theme).toBe('lab-dark');
    expect(window.localStorage.getItem('lab-theme')).toBe('lab-dark');

    fireEvent.click(screen.getByRole('button', { name: 'Use light theme' }));
    expect(document.documentElement.dataset.theme).toBe('lab-light');
    expect(window.localStorage.getItem('lab-theme')).toBe('lab-light');
  });
});
