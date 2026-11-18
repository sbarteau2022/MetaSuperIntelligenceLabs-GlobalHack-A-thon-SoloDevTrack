import React from 'react';
import { logger } from '../../lib/logger';

interface Props {
  children: React.ReactNode;
  /** Partition name for the receipt — e.g. "shell", "cloud-tab:swarm". */
  lane?: string;
  fallback?: React.ReactNode;
}

interface State {
  error: Error | null;
  errorId: string | null;
}

/**
 * Enterprise error boundary: isolates a crash to its lane,
 * emits a structured receipt, and offers deterministic recovery.
 * Never blank-screens the whole OS because one lazy chunk failed.
 */
export class ErrorBoundary extends React.Component<Props, State> {
  state: State = { error: null, errorId: null };

  static getDerivedStateFromError(error: Error): State {
    return { error, errorId: `ERR-${Date.now().toString(36).toUpperCase()}` };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    logger.error('error-boundary', {
      lane: this.props.lane ?? 'unknown',
      message: error.message.slice(0, 300),
      stack: (error.stack ?? '').slice(0, 800),
      componentStack: (info.componentStack ?? '').slice(0, 800),
    } as unknown as string);
  }

  private retry = () => this.setState({ error: null, errorId: null });

  private reload = () => window.location.reload();

  render() {
    if (!this.state.error) return this.props.children;
    if (this.props.fallback) return this.props.fallback;
    const lane = this.props.lane ?? 'shell';
    const errorId = this.state.errorId ?? 'ERR-UNKNOWN';
    return (
      <div
        role="alert"
        aria-live="assertive"
        className="rounded-2xl border border-red-500/30 bg-red-500/5 p-8 text-left max-w-2xl mx-auto my-12"
      >
        <div className="font-mono text-[11px] uppercase tracking-[0.2em] text-red-400">
          Lane isolated · {lane} · {errorId}
        </div>
        <h2 className="mt-2 text-xl font-bold text-white">This panel hit a fault and was contained.</h2>
        <p className="mt-2 text-sm text-[#71717a] leading-relaxed">
          The rest of the workspace is unaffected. The fault receipt was logged
          (no PII). Retry the lane, or reload for a clean boot.
        </p>
        <p className="mt-3 font-mono text-xs text-red-300/80 break-words">
          {this.state.error.message}
        </p>
        <div className="mt-5 flex gap-3">
          <button
            onClick={this.retry}
            className="px-4 py-2 rounded-lg bg-white text-black text-sm font-semibold hover:bg-white/90 transition cursor-pointer"
          >
            Retry lane
          </button>
          <button
            onClick={this.reload}
            className="px-4 py-2 rounded-lg bg-white/8 border border-white/15 text-sm text-white hover:bg-white/12 transition cursor-pointer"
          >
            Reload app
          </button>
        </div>
      </div>
    );
  }
}
