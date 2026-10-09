import { StatusBadge, type StatusTone } from '@/components/atoms/status-badge';
import type { AnalysisJobStatus } from '@/lib/api';

const STATUS_CONFIG: Record<AnalysisJobStatus, { label: string; tone: StatusTone }> = {
  queued: { label: 'Queued', tone: 'info' },
  running: { label: 'Running', tone: 'neutral' },
  succeeded: { label: 'Succeeded', tone: 'success' },
  failed: { label: 'Failed', tone: 'danger' },
  cancelled: { label: 'Cancelled', tone: 'warning' },
};

export interface AnalysisStatusBadgeProps {
  status: AnalysisJobStatus;
}

export interface AnalysisStatusDetailProps {
  status: AnalysisJobStatus;
  attemptCount: number;
  maxAttempts: number;
  error: { code: string; message: string } | null;
}

export const AnalysisStatusBadge = ({ status }: AnalysisStatusBadgeProps) => {
  const config = STATUS_CONFIG[status];
  return <StatusBadge tone={config.tone} label={config.label} />;
};

export const AnalysisStatusDetail = ({
  status,
  attemptCount,
  maxAttempts,
  error,
}: AnalysisStatusDetailProps) => {
  const config = STATUS_CONFIG[status];

  return (
    <div className='flex flex-col gap-2'>
      <div className='flex items-center gap-3'>
        <StatusBadge tone={config.tone} label={config.label} />
        {status === 'running' && (
          <span className='text-base-content/60 text-sm'>
            Attempt {attemptCount} of {maxAttempts}
          </span>
        )}
        {status === 'failed' && attemptCount >= maxAttempts && (
          <span className='text-error text-sm'>Max attempts reached</span>
        )}
      </div>
      {error && (
        <div className='text-error bg-error/10 rounded-md p-3 text-sm'>
          <strong>{error.code}</strong>: {error.message}
        </div>
      )}
    </div>
  );
};

export const analysisStatusIsTerminal = (status: AnalysisJobStatus): boolean => {
  return ['succeeded', 'failed', 'cancelled'].includes(status);
};

export const analysisStatusCanCancel = (status: AnalysisJobStatus): boolean => {
  return status === 'queued';
};
