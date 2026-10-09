import { OrganisationSwitcher } from '@/components/molecules/organisation-switcher';
import { SignOutButton } from '@/components/atoms/sign-out-button';
import { ThemeToggle } from '@/components/atoms/theme-toggle';
import type { MembershipSummary } from '@/lib/api';

export interface TopBarProps {
  memberships: MembershipSummary[];
  activeOrganisationId: string | null;
  userName: string;
}

export const TopBar = ({ memberships, activeOrganisationId, userName }: TopBarProps) => {
  const activeName =
    memberships.find((membership) => membership.organisation_id === activeOrganisationId)
      ?.organisation_name ?? 'No organisation';

  return (
    <header className='border-base-300 bg-base-100/95 sticky top-0 z-30 flex items-center justify-between gap-4 border-b px-4 py-3 backdrop-blur sm:px-6 lg:px-8'>
      <div className='flex items-center gap-3'>
        <span className='text-base-content/60 text-xs font-medium tracking-wide uppercase'>
          Organisation
        </span>
        {memberships.length > 1 ? (
          <OrganisationSwitcher
            memberships={memberships}
            activeOrganisationId={activeOrganisationId}
          />
        ) : (
          <span className='text-base-content text-sm font-semibold'>{activeName}</span>
        )}
      </div>

      <div className='flex items-center gap-3'>
        <ThemeToggle />
        <details className='dropdown dropdown-end'>
          <summary className='btn btn-ghost btn-sm list-none'>
            <span
              aria-hidden='true'
              className='bg-primary/10 text-primary flex h-7 w-7 items-center justify-center rounded-full text-xs font-semibold'>
              {userName.slice(0, 1).toUpperCase()}
            </span>
            {userName}
          </summary>
          <ul className='dropdown-content menu border-base-300 bg-base-100 rounded-box z-50 mt-3 w-48 border p-1 shadow-sm'>
            <li>
              <a href='/app/settings/profile'>Profile</a>
            </li>
            <li>
              <SignOutButton />
            </li>
          </ul>
        </details>
      </div>
    </header>
  );
};
