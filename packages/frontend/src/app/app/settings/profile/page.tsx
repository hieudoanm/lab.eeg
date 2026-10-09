import { Suspense } from 'react';

import { PageHeader } from '@/components/molecules/page-header';
import { LoadingState } from '@/components/atoms/states';
import { Card } from '@/components/atoms/card';
import { MetadataGrid } from '@/components/atoms/metadata-grid';
import { requireSession } from '@/trpc/server';

export const metadata = { title: 'Profile' };

const ProfileSettingsPage = () => {
  return (
    <div className='flex flex-col gap-6'>
      <PageHeader title='Profile' description='Your account details.' />
      <Suspense fallback={<LoadingState label='Loading your profile…' />}>
        <ProfileDetails />
      </Suspense>
    </div>
  );
};

const ProfileDetails = async () => {
  const { user } = await requireSession();

  return (
    <div className='flex flex-col gap-6'>
      <MetadataGrid
        items={[
          { label: 'Display name', value: user.display_name ?? 'Not set' },
          { label: 'Email', value: user.email ?? 'Not set' },
          { label: 'Account ID', value: user.id, mono: true },
        ]}
      />

      <Card title='Memberships' description='Organisations you can access and your role in each.'>
        {user.memberships.length === 0 ? (
          <p className='text-base-content/60 text-sm'>
            You are not a member of any organisation yet.
          </p>
        ) : (
          <ul className='divide-base-300 flex flex-col divide-y'>
            {user.memberships.map((membership) => (
              <li
                key={membership.membership_id}
                className='flex items-center justify-between gap-4 py-3 first:pt-0 last:pb-0'>
                <span className='text-base-content text-sm font-medium'>
                  {membership.organisation_name}
                </span>
                <span className='text-base-content/60 text-sm capitalize'>{membership.role}</span>
              </li>
            ))}
          </ul>
        )}
      </Card>
    </div>
  );
};

export default ProfileSettingsPage;
