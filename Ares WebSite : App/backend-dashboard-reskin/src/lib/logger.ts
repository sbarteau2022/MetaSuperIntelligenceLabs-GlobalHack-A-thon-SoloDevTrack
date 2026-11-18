import { config } from './config';

export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

const ORDER: Record<LogLevel, number> = { debug: 0, info: 1, warn: 2, error: 3 };

// Redact common secret shapes before they ever hit console/beacon.
const SECRET_RE = /(api[_-]?key|token|secret|authorization|bearer)\s*[:=]\s*\S+/gi;

function redact(arg: unknown): unknown {
  if (typeof arg === 'string') return arg.replace(SECRET_RE, '$1=[redacted]');
  return arg;
}

function enabled(level: LogLevel): boolean {
  return ORDER[level] >= ORDER[config.logLevel];
}

function emit(level: LogLevel, ...args: unknown[]) {
  if (!enabled(level)) return;
  const safe = args.map(redact);
  const prefix = `[${config.appName}] [${level}]`;
  if (level === 'error') console.error(prefix, ...safe);
  else if (level === 'warn') console.warn(prefix, ...safe);
  else if (level === 'info') console.info(prefix, ...safe);
  else console.debug(prefix, ...safe);
}

export const logger = {
  debug: (...a: unknown[]) => emit('debug', ...a),
  info: (...a: unknown[]) => emit('info', ...a),
  warn: (...a: unknown[]) => emit('warn', ...a),
  error: (...a: unknown[]) => emit('error', ...a),
  /** Structured event for future beacon transport (no PII). */
  event: (name: string, fields: Record<string, string | number | boolean> = {}) => {
    if (!config.telemetryEnabled) {
      emit('debug', 'event', name, fields);
      return;
    }
    emit('info', 'event', name, fields);
    try {
      const body = JSON.stringify({ name, fields, v: config.appVersion, t: Date.now() });
      if (navigator.sendBeacon) navigator.sendBeacon(config.telemetryEndpoint, body);
    } catch {
      /* never throw from telemetry */
    }
  },
};
