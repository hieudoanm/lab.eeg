import { describe, expect, it } from 'vitest';

import {
  formatBytes,
  formatChecksum,
  formatDateTime,
  formatDuration,
  formatNumber,
} from '../format';

describe('format', () => {
  describe('formatNumber', () => {
    it('formats finite numbers with grouping and bounded precision', () => {
      expect(formatNumber(0)).toBe('0');
      expect(formatNumber(1234.56789)).toBe('1,234.5679');
      expect(formatNumber(-12.5)).toBe('-12.5');
    });

    it('returns an em dash for nullish and non-finite values', () => {
      expect(formatNumber(null)).toBe('—');
      expect(formatNumber(undefined)).toBe('—');
      expect(formatNumber(Number.NaN)).toBe('—');
      expect(formatNumber(Number.POSITIVE_INFINITY)).toBe('—');
    });
  });

  describe('formatDuration', () => {
    it('returns an em dash for nullish and non-finite values', () => {
      expect(formatDuration(null)).toBe('—');
      expect(formatDuration(undefined)).toBe('—');
      expect(formatDuration(Number.NaN)).toBe('—');
      expect(formatDuration(Number.POSITIVE_INFINITY)).toBe('—');
    });

    it('formats sub-minute values in seconds', () => {
      expect(formatDuration(0)).toBe('0 s');
      expect(formatDuration(30)).toBe('30 s');
      expect(formatDuration(59.9)).toBe('59.9 s');
    });

    it('formats whole-minute values', () => {
      expect(formatDuration(60)).toBe('1 min 0 s');
      expect(formatDuration(90)).toBe('1 min 30 s');
      expect(formatDuration(3600)).toBe('60 min 0 s');
    });

    it('carries a rounded remainder into 60 seconds', () => {
      expect(formatDuration(119.6)).toBe('1 min 60 s');
    });
  });

  describe('formatBytes', () => {
    it('formats values in binary units', () => {
      expect(formatBytes(0)).toBe('0 B');
      expect(formatBytes(500)).toBe('500 B');
      expect(formatBytes(1024)).toBe('1 KiB');
      expect(formatBytes(1536)).toBe('1.5 KiB');
      expect(formatBytes(1024 * 1024)).toBe('1 MiB');
      expect(formatBytes(1024 * 1024 * 1024)).toBe('1 GiB');
    });

    it('caps the unit scale at GiB', () => {
      expect(formatBytes(1024 ** 4)).toBe('1,024 GiB');
    });

    it('returns an em dash for nullish and non-finite values', () => {
      expect(formatBytes(null)).toBe('—');
      expect(formatBytes(undefined)).toBe('—');
      expect(formatBytes(Number.NaN)).toBe('—');
    });
  });

  describe('formatChecksum', () => {
    it('returns an em dash for missing checksums', () => {
      expect(formatChecksum(null)).toBe('—');
      expect(formatChecksum(undefined)).toBe('—');
      expect(formatChecksum('')).toBe('—');
    });

    it('keeps short checksums intact', () => {
      expect(formatChecksum('abc123')).toBe('abc123');
      expect(formatChecksum('a'.repeat(16))).toBe('a'.repeat(16));
    });

    it('truncates long checksums with an ellipsis', () => {
      expect(formatChecksum('a'.repeat(64))).toBe(`${'a'.repeat(12)}…`);
    });
  });

  describe('formatDateTime', () => {
    it('returns an em dash for missing values', () => {
      expect(formatDateTime(null)).toBe('—');
      expect(formatDateTime(undefined)).toBe('—');
    });

    it('returns an em dash for an unparseable date', () => {
      expect(formatDateTime('not-a-date')).toBe('—');
    });

    it('formats a valid timestamp in UTC', () => {
      expect(formatDateTime('2025-01-01T00:00:00.000Z')).toBe('1 Jan 2025, 00:00 UTC');
      expect(formatDateTime('2025-06-15T13:45:00.000Z')).toBe('15 Jun 2025, 13:45 UTC');
    });
  });
});
