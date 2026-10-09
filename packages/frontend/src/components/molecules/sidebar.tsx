'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Suspense, useState, type ReactNode } from 'react';

import { cn } from '@/lib/cn';

interface NavItem {
  href: string;
  label: string;
  icon: ReactNode;
}

const IconDashboard = () => {
  return (
    <svg viewBox='0 0 24 24' fill='none' stroke='currentColor' strokeWidth='1.7' aria-hidden='true'>
      <path d='M3 12l9-9 9 9' />
      <path d='M5 10v10h14V10' />
    </svg>
  );
};

const IconStudies = () => {
  return (
    <svg viewBox='0 0 24 24' fill='none' stroke='currentColor' strokeWidth='1.7' aria-hidden='true'>
      <path d='M4 5a2 2 0 0 1 2-2h9l5 5v11a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2z' />
      <path d='M14 3v6h6' />
    </svg>
  );
};

const IconAnalyses = () => {
  return (
    <svg viewBox='0 0 24 24' fill='none' stroke='currentColor' strokeWidth='1.7' aria-hidden='true'>
      <path d='M4 19V5' />
      <path d='M4 19h16' />
      <path d='M8 15l3-4 3 3 5-7' />
    </svg>
  );
};

const IconSettings = () => {
  return (
    <svg viewBox='0 0 24 24' fill='none' stroke='currentColor' strokeWidth='1.7' aria-hidden='true'>
      <circle cx='12' cy='12' r='3' />
      <path d='M12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M19 5l-2 2M7 17l-2 2' />
    </svg>
  );
};

const NAV_ITEMS: NavItem[] = [
  { href: '/app/dashboard', label: 'Dashboard', icon: <IconDashboard /> },
  { href: '/app/studies', label: 'Studies', icon: <IconStudies /> },
  { href: '/app/analyses', label: 'Analyses', icon: <IconAnalyses /> },
  { href: '/app/settings/organisation', label: 'Organisation', icon: <IconSettings /> },
];

const isActive = (pathname: string, href: string): boolean => {
  return pathname === href || pathname.startsWith(`${href}/`);
};

const NavLinks = ({
  pathname,
  onNavigate,
}: {
  pathname: string | null;
  onNavigate?: () => void;
}) => {
  return (
    <ul className='menu w-full gap-1 p-0'>
      {NAV_ITEMS.map((item) => {
        const active = pathname !== null && isActive(pathname, item.href);
        return (
          <li key={item.href}>
            <Link
              href={item.href}
              onClick={onNavigate}
              aria-current={active ? 'page' : undefined}
              className={cn(
                active
                  ? 'bg-primary/10 text-primary font-medium'
                  : 'text-base-content/60 hover:bg-base-200 hover:text-base-content',
              )}>
              <span className='h-5 w-5 shrink-0'>{item.icon}</span>
              {item.label}
            </Link>
          </li>
        );
      })}
    </ul>
  );
};

const ActiveNavLinks = ({ onNavigate }: { onNavigate: () => void }) => {
  const pathname = usePathname();
  return <NavLinks pathname={pathname} onNavigate={onNavigate} />;
};

export const Sidebar = () => {
  const [open, setOpen] = useState(false);

  return (
    <>
      <button
        type='button'
        onClick={() => setOpen(true)}
        className='btn btn-outline btn-sm lg:hidden'
        aria-expanded={open}
        aria-controls='app-sidebar'>
        <span aria-hidden='true'>☰</span>
        Menu
      </button>

      {open ? (
        <div
          className='bg-base-content/30 fixed inset-0 z-40 lg:hidden'
          onClick={() => setOpen(false)}
          aria-hidden='true'
        />
      ) : null}

      <aside
        id='app-sidebar'
        className={cn(
          'border-base-300 bg-base-100 fixed inset-y-0 left-0 z-50 w-64 border-r p-4 transition-transform lg:static lg:z-auto lg:translate-x-0',
          open ? 'translate-x-0' : '-translate-x-full lg:translate-x-0',
        )}>
        <div className='mb-6 flex items-center justify-between'>
          <Link href='/app/dashboard' className='flex items-center gap-2'>
            <span aria-hidden='true' className='bg-primary h-6 w-6 rounded' />
            <span className='text-base-content text-base font-semibold tracking-tight'>
              EEG Lab
            </span>
          </Link>
          <button
            type='button'
            onClick={() => setOpen(false)}
            className='btn btn-ghost btn-sm lg:hidden'
            aria-label='Close navigation'>
            ×
          </button>
        </div>
        <nav aria-label='Primary'>
          <Suspense fallback={<NavLinks pathname={null} />}>
            <ActiveNavLinks onNavigate={() => setOpen(false)} />
          </Suspense>
        </nav>
      </aside>
    </>
  );
};
