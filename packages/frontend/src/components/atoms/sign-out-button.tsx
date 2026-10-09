'use client';

import { useRouter } from 'next/navigation';
import { useState } from 'react';

import { api } from '@/trpc/react';

export const SignOutButton = () => {
  const router = useRouter();
  const [error, setError] = useState<string | null>(null);
  const signOut = api.identity.signOut.useMutation({
    onSuccess: () => {
      router.push('/login');
      router.refresh();
    },
    onError: () => {
      setError('Could not sign out. Please try again.');
    },
  });

  const handleSignOut = () => {
    setError(null);
    signOut.mutate();
  };

  return (
    <div>
      <button
        type='button'
        onClick={handleSignOut}
        disabled={signOut.isPending}
        className='btn btn-ghost btn-sm w-full justify-start font-normal disabled:opacity-60'>
        {signOut.isPending ? 'Signing out…' : 'Sign out'}
      </button>
      {error ? (
        <p role='alert' className='text-error text-xs'>
          {error}
        </p>
      ) : null}
    </div>
  );
};
