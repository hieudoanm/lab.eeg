import type { ReactNode } from 'react';

export interface MetadataItem {
  label: string;
  value: ReactNode;
  mono?: boolean;
}

export interface MetadataGridProps {
  items: MetadataItem[];
  columns?: 1 | 2 | 3;
}

const COLUMNS: Record<1 | 2 | 3, string> = {
  1: 'grid-cols-1',
  2: 'grid-cols-1 sm:grid-cols-2',
  3: 'grid-cols-1 sm:grid-cols-2 lg:grid-cols-3',
};

export const MetadataGrid = ({ items, columns = 2 }: MetadataGridProps) => {
  return (
    <dl className={`grid gap-x-6 gap-y-4 ${COLUMNS[columns]}`}>
      {items.map((item) => (
        <div key={item.label} className='flex flex-col gap-0.5'>
          <dt className='text-base-content/60 text-xs font-medium tracking-wide uppercase'>
            {item.label}
          </dt>
          <dd
            className={
              item.mono ? 'text-base-content font-mono text-sm' : 'text-base-content text-sm'
            }>
            {item.value}
          </dd>
        </div>
      ))}
    </dl>
  );
};
