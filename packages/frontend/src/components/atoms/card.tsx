import type { ReactNode } from 'react';

import { cn } from '@/lib/cn';

export interface CardProps {
  children: ReactNode;
  className?: string;
  title?: string;
  description?: string;
  actions?: ReactNode;
}

export const Card = ({ children, className, title, description, actions }: CardProps) => {
  return (
    <section className={cn('card border-base-300 bg-base-100 border', className)}>
      {title ? (
        <header className='border-base-300 flex items-start justify-between gap-4 border-b px-5 py-4'>
          <div>
            <h2 className='card-title text-base font-semibold'>{title}</h2>
            {description ? <p className='text-base-content/60 text-sm'>{description}</p> : null}
          </div>
          {actions}
        </header>
      ) : null}
      <div className='p-5'>{children}</div>
    </section>
  );
};
