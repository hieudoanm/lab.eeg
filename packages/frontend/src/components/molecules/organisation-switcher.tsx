'use client';

import { useRouter } from 'next/navigation';
import { useState } from 'react';

import type { MembershipSummary } from '@/lib/api';
import { api } from '@/trpc/react';

export interface OrganisationSwitcherProps {
  memberships: MembershipSummary[];
  activeOrganisationId: string | null;
}

export const OrganisationSwitcher = ({
  memberships,
  activeOrganisationId,
}: OrganisationSwitcherProps) => {
  const router = useRouter();
  const [error, setError] = useState<string | null>(null);
  const selectOrganisation = api.identity.selectOrganisation.useMutation({
    onSuccess: () => {
      router.refresh();
    },
    onError: () => {
      setError('Could not switch organisation. Please try again.');
    },
  });

  const handleChange = (organisationId: string) => {
    setError(null);
    selectOrganisation.mutate({ organisationId });
  };

  return (
    <div className='flex flex-col gap-1'>
      <label className='flex items-center gap-2'>
        <span className='sr-only'>Active organisation</span>
        <select
          value={activeOrganisationId ?? ''}
          onChange={(event) => handleChange(event.target.value)}
          disabled={selectOrganisation.isPending}
          className='select select-sm w-auto font-semibold disabled:opacity-60'>
          {memberships.map((membership) => (
            <option key={membership.organisation_id} value={membership.organisation_id}>
              {membership.organisation_name}
            </option>
          ))}
        </select>
      </label>
      {error ? (
        <p role='alert' className='text-error text-xs'>
          {error}
        </p>
      ) : null}
    </div>
  );
};
