import { describe, expect, it } from 'vitest';
import { formatBytes, formatResultJson, runProgressLabel } from './format';

describe('formatResultJson', () => {
  it('pretty-prints valid JSON', () => {
    expect(formatResultJson('{"a":1}')).toBe('{\n  "a": 1\n}');
  });
  it('falls back to truncated raw text for invalid JSON', () => {
    expect(formatResultJson('not json')).toBe('not json');
    expect(formatResultJson('x'.repeat(5000))).toHaveLength(4000);
  });
});

describe('formatBytes', () => {
  it('formats byte units', () => {
    expect(formatBytes(512)).toBe('512 B');
    expect(formatBytes(2048)).toBe('2.0 KiB');
    expect(formatBytes(5 * 1024 * 1024)).toBe('5.0 MiB');
  });
});

describe('runProgressLabel', () => {
  it('shows status for terminal runs', () => {
    expect(runProgressLabel({ status: 'succeeded' })).toBe('succeeded');
  });
  it('shows percent and message for running runs', () => {
    expect(
      runProgressLabel({ status: 'running', progress: 0.42, progress_message: 'filtering' }),
    ).toBe('42% · filtering');
    expect(runProgressLabel({ status: 'running', progress: 0.5 })).toBe('50%');
  });
});
