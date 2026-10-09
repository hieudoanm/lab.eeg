import type { ReactNode } from 'react';

import { cn } from '@/lib/cn';

export interface LoadingStateProps {
  label?: string;
  className?: string;
}

export const LoadingState = ({ label = 'Loading…', className }: LoadingStateProps) => {
  return (
    <div
      role='status'
      aria-live='polite'
      className={cn('text-base-content/60 flex items-center gap-3 p-6 text-sm', className)}>
      <span aria-hidden='true' className='loading loading-spinner loading-sm' />
      {label}
    </div>
  );
};

export interface EmptyStateProps {
  title: string;
  description: string;
  action?: ReactNode;
  className?: string;
}

export const EmptyState = ({ title, description, action, className }: EmptyStateProps) => {
  return (
    <div
      className={cn(
        'border-base-300 flex flex-col items-center gap-2 rounded-[var(--radius-card)] border border-dashed p-10 text-center',
        className,
      )}>
      <h3 className='text-base-content text-base font-semibold'>{title}</h3>
      <p className='text-base-content/60 max-w-md text-sm'>{description}</p>
      {action ? <div className='mt-2'>{action}</div> : null}
    </div>
  );
};

export interface ErrorStateProps {
  title?: string;
  message: string;
  requestId?: string | null;
  action?: ReactNode;
  className?: string;
}

export const ErrorState = ({
  title = 'Something went wrong',
  message,
  requestId,
  action,
  className,
}: ErrorStateProps) => {
  return (
    <div role='alert' className={cn('alert alert-soft alert-error text-start', className)}>
      <div className='flex flex-col gap-2'>
        <h3 className='text-base font-semibold'>{title}</h3>
        <p className='text-sm'>{message}</p>
        {requestId ? <p className='font-mono text-xs'>Request ID: {requestId}</p> : null}
        {action ? <div className='mt-2'>{action}</div> : null}
      </div>
    </div>
  );
};

export interface NoResultsStateProps {
  description?: string;
  className?: string;
}

export const NoResultsState = ({
  description = 'No resources match the current filters.',
  className,
}: NoResultsStateProps) => {
  return (
    <div
      role='status'
      className={cn(
        'border-base-300 bg-base-100 text-base-content/60 rounded-[var(--radius-card)] border p-8 text-center text-sm',
        className,
      )}>
      {description}
    </div>
  );
};
