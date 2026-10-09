import { describe, expect, it } from 'vitest';

import { cn } from '../cn';

describe('cn', () => {
  it('joins non-empty class names', () => {
    expect(cn('a', 'b')).toBe('a b');
  });

  it('filters falsy values', () => {
    expect(cn('a', false, null, undefined, 'b')).toBe('a b');
  });

  it('filters empty strings', () => {
    expect(cn('', 'a', '', 'b')).toBe('a b');
  });

  it('returns empty string when no classes', () => {
    expect(cn(false, null, undefined)).toBe('');
  });

  it('returns empty string with no arguments', () => {
    expect(cn()).toBe('');
  });
});
