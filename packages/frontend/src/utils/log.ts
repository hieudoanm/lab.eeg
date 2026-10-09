export const LOG_LEVELS = ['trace', 'debug', 'info', 'warn', 'error'] as const;

export type LogLevel = (typeof LOG_LEVELS)[number];
export type LogContext = Record<string, unknown>;

export interface Logger {
  trace(message: string, context?: LogContext): void;
  debug(message: string, context?: LogContext): void;
  info(message: string, context?: LogContext): void;
  warn(message: string, context?: LogContext): void;
  error(message: string, context?: LogContext): void;
}

export interface LoggerOptions {
  level?: LogLevel;
}

const LEVEL_PRIORITY: Record<LogLevel, number> = {
  trace: 10,
  debug: 20,
  info: 30,
  warn: 40,
  error: 50,
};

const SENSITIVE_KEY = /(?:authorization|cookie|password|secret|token|credential|api[_-]?key)/i;

const configuredLogLevel = (): LogLevel => {
  const configured = process.env.NEXT_PUBLIC_LOG_LEVEL;
  if (configured && LOG_LEVELS.includes(configured as LogLevel)) {
    return configured as LogLevel;
  }
  return process.env.NODE_ENV === 'production' ? 'info' : 'debug';
};

const normalizeValue = (value: unknown, seen: WeakSet<object>): unknown => {
  if (typeof value === 'bigint') {
    return value.toString();
  }
  if (value instanceof Error) {
    return {
      name: value.name,
      message: value.message,
      stack: value.stack,
    };
  }
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value !== 'object' || value === null) {
    return value;
  }
  if (seen.has(value)) {
    return '[Circular]';
  }
  seen.add(value);

  if (Array.isArray(value)) {
    const normalized = value.map((item) => normalizeValue(item, seen));
    seen.delete(value);
    return normalized;
  }

  const normalized = Object.fromEntries(
    Object.entries(value).map(([key, entry]) => [
      key,
      SENSITIVE_KEY.test(key) ? '[REDACTED]' : normalizeValue(entry, seen),
    ]),
  );
  seen.delete(value);
  return normalized;
};

const writeLog = (
  level: LogLevel,
  service: string,
  message: string,
  context?: LogContext,
): void => {
  const entry = {
    timestamp: new Date().toISOString(),
    level,
    service,
    message,
    ...(context && Object.keys(context).length > 0
      ? { context: normalizeValue(context, new WeakSet()) }
      : {}),
  };
  const output = JSON.stringify(entry);

  if (level === 'trace' || level === 'debug') {
    console.debug(output);
  } else if (level === 'info') {
    console.info(output);
  } else if (level === 'warn') {
    console.warn(output);
  } else {
    console.error(output);
  }
};

export const createLogger = (service: string, options: LoggerOptions = {}): Logger => {
  const minimumPriority = LEVEL_PRIORITY[options.level ?? configuredLogLevel()];
  const write =
    (level: LogLevel) =>
    (message: string, context?: LogContext): void => {
      if (LEVEL_PRIORITY[level] >= minimumPriority) {
        writeLog(level, service, message, context);
      }
    };

  return {
    trace: write('trace'),
    debug: write('debug'),
    info: write('info'),
    warn: write('warn'),
    error: write('error'),
  };
};

export const log = createLogger('eeg-lab-web');
