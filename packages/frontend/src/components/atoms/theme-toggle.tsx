'use client';

import { cn } from '@/lib/cn';

import { useTheme, type LabTheme } from './theme-provider';

const SunIcon = ({ className }: { className?: string }) => (
  <svg
    viewBox='0 0 24 24'
    fill='none'
    stroke='currentColor'
    strokeWidth='1.8'
    strokeLinecap='round'
    strokeLinejoin='round'
    className={className}
    aria-hidden='true'>
    <circle cx='12' cy='12' r='4' />
    <path d='M12 2v2m0 16v2M4.9 4.9l1.4 1.4m11.4 11.4 1.4 1.4M2 12h2m16 0h2M4.9 19.1l1.4-1.4m11.4-11.4 1.4-1.4' />
  </svg>
);

const MoonIcon = ({ className }: { className?: string }) => (
  <svg
    viewBox='0 0 24 24'
    fill='none'
    stroke='currentColor'
    strokeWidth='1.8'
    strokeLinecap='round'
    strokeLinejoin='round'
    className={className}
    aria-hidden='true'>
    <path d='M20 14.5A8 8 0 0 1 9.5 4 8 8 0 1 0 20 14.5Z' />
  </svg>
);

/** Toggle between the Lab light and dark themes. */
export const ThemeToggle = ({ className }: { className?: string }) => {
  const { theme, setTheme } = useTheme();

  const option = (label: string, value: LabTheme, icon: React.ReactNode) => {
    const active = theme === value;
    return (
      <button
        type='button'
        aria-pressed={active}
        aria-label={`Use ${label} theme`}
        title={`${label} theme`}
        onClick={() => setTheme(value)}
        className={cn(
          'text-base-content/60 flex h-7 w-7 items-center justify-center rounded-md transition-colors',
          active && 'bg-base-200 text-base-content',
        )}>
        {icon}
      </button>
    );
  };

  return (
    <div
      role='group'
      aria-label='Colour theme'
      className={cn(
        'border-base-300 bg-base-100 flex items-center rounded-lg border p-0.5',
        className,
      )}>
      {option('light', 'lab-light', <SunIcon className='h-4 w-4' />)}
      {option('dark', 'lab-dark', <MoonIcon className='h-4 w-4' />)}
    </div>
  );
};
