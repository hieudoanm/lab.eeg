import { cn } from '@/lib/cn';

export type StatusTone = 'neutral' | 'info' | 'success' | 'warning' | 'danger';

const TONES: Record<StatusTone, string> = {
  neutral: 'badge-neutral',
  info: 'badge-info',
  success: 'badge-success',
  warning: 'badge-warning',
  danger: 'badge-error',
};

const DOT: Record<StatusTone, string> = {
  neutral: 'bg-base-content/50',
  info: 'bg-info',
  success: 'bg-success',
  warning: 'bg-warning',
  danger: 'bg-error',
};

export interface StatusBadgeProps {
  tone: StatusTone;
  label: string;
  className?: string;
}

/**
 * A status indicator that pairs colour with a text label and a shape marker so
 * meaning is never conveyed by colour alone (docs/09-design.md §17).
 */
export const StatusBadge = ({ tone, label, className }: StatusBadgeProps) => {
  return (
    <span className={cn('badge badge-soft gap-1.5 font-medium', TONES[tone], className)}>
      <span aria-hidden='true' className={cn('h-1.5 w-1.5 rounded-full', DOT[tone])} />
      {label}
    </span>
  );
};
