import { afterEach, describe, expect, it } from 'vitest';

import { getAccessToken, setAccessToken } from '../token';

afterEach(() => {
  setAccessToken(null);
});

describe('access token holder', () => {
  it('round-trips a stored token', () => {
    setAccessToken('token-abc');
    expect(getAccessToken()).toBe('token-abc');
  });

  it('replaces a previously stored token', () => {
    setAccessToken('first');
    setAccessToken('second');
    expect(getAccessToken()).toBe('second');
  });

  it('clears the token when set to null', () => {
    setAccessToken('token-abc');
    setAccessToken(null);
    expect(getAccessToken()).toBeNull();
  });
});
