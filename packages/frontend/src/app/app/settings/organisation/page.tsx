import { Suspense } from 'react';

import { PageHeader } from '@/components/molecules/page-header';
import { EmptyState, ErrorState, LoadingState } from '@/components/atoms/states';
import { Card } from '@/components/atoms/card';
import { MetadataGrid } from '@/components/atoms/metadata-grid';
import type { Membership, OrganisationDetail } from '@/lib/api';
import { describeError } from '@/trpc/errors';
import { requireSession } from '@/trpc/server';

export const metadata = { title: 'Organisation settings' };

const OrganisationSettingsPage = () => {
  return (
    <div className='flex flex-col gap-6'>
      <PageHeader
        title='Organisation settings'
        description='Members and roles for the active organisation.'
      />
      <Suspense fallback={<LoadingState label='Loading organisation…' />}>
        <OrganisationDetails />
      </Suspense>
    </div>
  );
};

const OrganisationDetails = async () => {
  const { caller, membership } = await requireSession();
  if (!membership) {
    return (
      <EmptyState
        title='No organisation'
        description='You are not a member of any organisation yet.'
      />
    );
  }

  let organisation: OrganisationDetail | null = null;
  let members: Membership[] = [];
  let errorMessage: string | null = null;
  let requestId: string | null = null;
  try {
    const [organisationResult, membersResult] = await Promise.all([
      caller.identity.organisation(),
      caller.identity.members(),
    ]);
    organisation = organisationResult;
    members = membersResult.items;
  } catch (error) {
    const described = describeError(error, 'The organisation could not be loaded.');
    errorMessage = described.message;
    requestId = described.requestId;
  }

  if (errorMessage !== null || organisation === null) {
    return (
      <ErrorState
        message={errorMessage ?? 'The organisation could not be loaded.'}
        requestId={requestId}
      />
    );
  }

  return (
    <div className='flex flex-col gap-6'>
      <MetadataGrid
        items={[
          { label: 'Name', value: organisation.name },
          { label: 'Status', value: organisation.status },
          { label: 'Your role', value: organisation.role },
          { label: 'Organisation ID', value: organisation.id, mono: true },
        ]}
      />

      <Card
        title='Members'
        description={`${members.length} member${members.length === 1 ? '' : 's'}`}>
        {members.length === 0 ? (
          <p className='text-base-content/60 text-sm'>This organisation has no members.</p>
        ) : (
          <ul className='divide-base-300 flex flex-col divide-y'>
            {members.map((member) => (
              <li
                key={member.id}
                className='flex flex-wrap items-center justify-between gap-2 py-3 first:pt-0 last:pb-0'>
                <div className='flex flex-col'>
                  <span className='text-base-content text-sm font-medium'>
                    {member.display_name ?? 'Unnamed member'}
                  </span>
                  {member.email ? (
                    <span className='text-base-content/60 text-xs'>{member.email}</span>
                  ) : null}
                </div>
                <span className='text-base-content/60 text-sm capitalize'>{member.role}</span>
              </li>
            ))}
          </ul>
        )}
      </Card>
    </div>
  );
};

export default OrganisationSettingsPage;
