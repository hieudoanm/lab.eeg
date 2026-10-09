import type { ReactNode } from 'react';

import { Breadcrumbs, type Breadcrumb } from '@/components/atoms/breadcrumbs';

export interface PageHeaderProps {
  title: string;
  description?: string;
  breadcrumbs?: Breadcrumb[];
  actions?: ReactNode;
  children?: ReactNode;
}

export const PageHeader = ({
  title,
  description,
  breadcrumbs,
  actions,
  children,
}: PageHeaderProps) => {
  return (
    <header className='border-base-300 flex flex-col gap-4 border-b pb-6'>
      {breadcrumbs ? <Breadcrumbs items={breadcrumbs} /> : null}
      <div className='flex flex-wrap items-start justify-between gap-4'>
        <div>
          <h1 className='text-base-content text-2xl font-semibold tracking-tight sm:text-3xl'>
            {title}
          </h1>
          {description ? <p className='text-base-content/60 text-sm'>{description}</p> : null}
        </div>
        {actions ? <div className='flex items-center gap-2'>{actions}</div> : null}
      </div>
      {children}
    </header>
  );
};
