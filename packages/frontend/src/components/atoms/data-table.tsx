import type { ReactNode } from 'react';

import { cn } from '@/lib/cn';

export interface DataTableProps {
  columns: ReactNode[];
  children: ReactNode;
  className?: string;
}

export const DataTable = ({ columns, children, className }: DataTableProps) => {
  return (
    <div className={cn('w-full overflow-x-auto', className)}>
      <table className='table'>
        <thead>
          <tr className='border-base-300 border-b text-left'>
            {columns.map((column, index) => (
              <th
                key={index}
                scope='col'
                className='text-base-content/60 px-3 py-2 font-medium whitespace-nowrap'>
                {column}
              </th>
            ))}
          </tr>
        </thead>
        <tbody className='divide-base-300 divide-y'>{children}</tbody>
      </table>
    </div>
  );
};
