import { Suspense } from 'react';

import { AppShell } from '@/components/organisms/app-shell';
import { TopBar } from '@/components/molecules/top-bar';
import { requireSession } from '@/trpc/server';

const AppLayout = ({ children }: LayoutProps<'/app'>) => {
  return (
    <AppShell
      topBar={
        <Suspense fallback={<TopBarFallback />}>
          <SessionTopBar />
        </Suspense>
      }>
      {children}
    </AppShell>
  );
};

const SessionTopBar = async () => {
  const { user, membership } = await requireSession();
  return (
    <TopBar
      memberships={user.memberships}
      activeOrganisationId={membership?.organisation_id ?? null}
      userName={user.display_name ?? user.email ?? 'Account'}
    />
  );
};

const TopBarFallback = () => {
  return (
    <header
      aria-hidden='true'
      className='border-base-300 bg-base-100/95 sticky top-0 z-30 flex items-center justify-between gap-4 border-b px-4 py-3 backdrop-blur sm:px-6 lg:px-8'>
      <span className='bg-base-200 h-4 w-40 animate-pulse rounded' />
      <span className='bg-base-200 h-7 w-28 animate-pulse rounded' />
    </header>
  );
};

export default AppLayout;
