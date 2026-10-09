import { afterEach, describe, expect, it, vi } from 'vitest';

import { createLogger } from '../log';

interface LogEntry {
  timestamp: string;
  level: string;
  service: string;
  message: string;
  context?: Record<string, unknown>;
}

const readEntry = (spy: { mock: { calls: unknown[][] } }): LogEntry => {
  const output = spy.mock.calls[0][0] as string;
  return JSON.parse(output) as LogEntry;
};

const silence = () => {
  vi.spyOn(console, 'debug').mockImplementation(() => {});
  vi.spyOn(console, 'info').mockImplementation(() => {});
  vi.spyOn(console, 'warn').mockImplementation(() => {});
  return vi.spyOn(console, 'error').mockImplementation(() => {});
};

afterEach(() => {
  vi.unstubAllEnvs();
  vi.restoreAllMocks();
});

describe('createLogger', () => {
  it('logs at or above the configured level', () => {
    const spy = vi.spyOn(console, 'debug').mockImplementation(() => {});
    const logger = createLogger('test', { level: 'debug' });
    logger.debug('message', { a: 1 });
    expect(spy).toHaveBeenCalled();
  });

  it('respects log level filtering', () => {
    const spy = vi.spyOn(console, 'debug').mockImplementation(() => {});
    const logger = createLogger('test', { level: 'info' });
    logger.debug('skip');
    expect(spy).not.toHaveBeenCalled();
  });

  it('suppresses info when the minimum level is warn', () => {
    const infoSpy = vi.spyOn(console, 'info').mockImplementation(() => {});
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    const logger = createLogger('test', { level: 'warn' });

    logger.info('skip');
    logger.warn('keep');

    expect(infoSpy).not.toHaveBeenCalled();
    expect(warnSpy).toHaveBeenCalledTimes(1);
  });

  it('routes each level to its matching console method', () => {
    const debugSpy = vi.spyOn(console, 'debug').mockImplementation(() => {});
    const infoSpy = vi.spyOn(console, 'info').mockImplementation(() => {});
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const logger = createLogger('svc', { level: 'trace' });

    logger.trace('t');
    logger.debug('d');
    logger.info('i');
    logger.warn('w');
    logger.error('e');

    expect(debugSpy).toHaveBeenCalledTimes(2);
    expect(infoSpy).toHaveBeenCalledTimes(1);
    expect(warnSpy).toHaveBeenCalledTimes(1);
    expect(errorSpy).toHaveBeenCalledTimes(1);
  });

  it('writes a structured entry with service, level, and message', () => {
    const infoSpy = vi.spyOn(console, 'info').mockImplementation(() => {});
    const logger = createLogger('svc', { level: 'debug' });

    logger.info('hello', { count: 2 });

    const entry = readEntry(infoSpy);
    expect(entry.level).toBe('info');
    expect(entry.service).toBe('svc');
    expect(entry.message).toBe('hello');
    expect(entry.context).toEqual({ count: 2 });
    expect(typeof entry.timestamp).toBe('string');
  });

  it('omits the context key when no context is provided', () => {
    const infoSpy = vi.spyOn(console, 'info').mockImplementation(() => {});
    const logger = createLogger('svc', { level: 'debug' });

    logger.info('bare');
    logger.info('empty', {});

    expect(readEntry(infoSpy)).not.toHaveProperty('context');
  });

  it('uses the environment log level when valid', () => {
    vi.stubEnv('NEXT_PUBLIC_LOG_LEVEL', 'error');
    silence();
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const logger = createLogger('env');

    logger.warn('skip');
    logger.error('keep');

    expect(warnSpy).not.toHaveBeenCalled();
    expect(errorSpy).toHaveBeenCalledTimes(1);
  });

  it('falls back to debug for an invalid log level', () => {
    vi.stubEnv('NEXT_PUBLIC_LOG_LEVEL', 'verbose');
    const debugSpy = vi.spyOn(console, 'debug').mockImplementation(() => {});
    const logger = createLogger('env');

    logger.debug('keep');

    expect(debugSpy).toHaveBeenCalledTimes(1);
  });

  it('defaults to info in production', () => {
    vi.stubEnv('NEXT_PUBLIC_LOG_LEVEL', '');
    vi.stubEnv('NODE_ENV', 'production');
    const debugSpy = vi.spyOn(console, 'debug').mockImplementation(() => {});
    const infoSpy = vi.spyOn(console, 'info').mockImplementation(() => {});
    const logger = createLogger('env');

    logger.debug('skip');
    logger.info('keep');

    expect(debugSpy).not.toHaveBeenCalled();
    expect(infoSpy).toHaveBeenCalledTimes(1);
  });

  it('redacts sensitive context keys', () => {
    const infoSpy = vi.spyOn(console, 'info').mockImplementation(() => {});
    const logger = createLogger('svc', { level: 'debug' });

    logger.info('secret', {
      authorization: 'Bearer secret',
      token: 'tok',
      password: 'hunter2',
      secret: 's3cret',
      cookie: 'session=abc',
      api_key: 'key',
      safe: 'visible',
    });

    const entry = readEntry(infoSpy);
    const context = entry.context as Record<string, unknown>;
    expect(context.authorization).toBe('[REDACTED]');
    expect(context.token).toBe('[REDACTED]');
    expect(context.password).toBe('[REDACTED]');
    expect(context.secret).toBe('[REDACTED]');
    expect(context.cookie).toBe('[REDACTED]');
    expect(context.api_key).toBe('[REDACTED]');
    expect(context.safe).toBe('visible');
    expect(JSON.stringify(entry)).not.toContain('hunter2');
  });

  it('normalises errors, dates, and bigints', () => {
    const infoSpy = vi.spyOn(console, 'info').mockImplementation(() => {});
    const logger = createLogger('svc', { level: 'debug' });

    logger.info('values', {
      err: new Error('boom'),
      when: new Date('2020-01-02T03:04:05.000Z'),
      big: BigInt(10),
    });

    const context = readEntry(infoSpy).context as Record<string, unknown>;
    expect(context.err).toMatchObject({ name: 'Error', message: 'boom' });
    expect(typeof (context.err as { stack: unknown }).stack).toBe('string');
    expect(context.when).toBe('2020-01-02T03:04:05.000Z');
    expect(context.big).toBe('10');
  });

  it('normalises arrays and nested objects', () => {
    const infoSpy = vi.spyOn(console, 'info').mockImplementation(() => {});
    const logger = createLogger('svc', { level: 'debug' });

    logger.info('nested', {
      list: [1, new Date('2020-01-02T03:04:05.000Z'), { token: 'x' }],
      outer: { inner: { password: 'y' } },
    });

    const context = readEntry(infoSpy).context as Record<string, unknown>;
    const list = context.list as unknown[];
    expect(list[0]).toBe(1);
    expect(list[1]).toBe('2020-01-02T03:04:05.000Z');
    expect((list[2] as Record<string, unknown>).token).toBe('[REDACTED]');
    expect(
      ((context.outer as Record<string, unknown>).inner as Record<string, unknown>).password,
    ).toBe('[REDACTED]');
  });

  it('collapses circular references', () => {
    const infoSpy = vi.spyOn(console, 'info').mockImplementation(() => {});
    const logger = createLogger('svc', { level: 'debug' });

    const node: Record<string, unknown> = { name: 'root' };
    node.self = node;

    logger.info('cycle', { node });

    const context = readEntry(infoSpy).context as Record<string, unknown>;
    expect((context.node as Record<string, unknown>).self).toBe('[Circular]');
    expect((context.node as Record<string, unknown>).name).toBe('root');
  });
});
