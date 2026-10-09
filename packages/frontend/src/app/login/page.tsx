import type { Metadata, NextPage } from 'next';

import { ThemeToggle } from '@/components/atoms/theme-toggle';
import { LoginForm } from '@/components/molecules/login-form';

export const metadata: Metadata = { title: 'Sign in' };

const LoginPage: NextPage = () => {
  return (
    <main className='bg-background relative flex min-h-screen items-center justify-center px-4 py-12'>
      <div className='absolute top-4 right-4'>
        <ThemeToggle />
      </div>
      <div className='w-full max-w-sm'>
        <div className='mb-6 flex items-center gap-2'>
          <span aria-hidden='true' className='bg-primary h-6 w-6 rounded' />
          <span className='text-base-content text-lg font-semibold tracking-tight'>EEG Lab</span>
        </div>

        <div className='border-base-300 bg-base-100 rounded-[var(--radius-card)] border p-6'>
          <h1 className='text-base-content text-xl font-semibold'>Sign in</h1>
          <p className='text-base-content/60 mt-1 text-sm'>
            Access your organisation&apos;s research workspace.
          </p>

          <LoginForm />

          <p className='text-base-content/60 mt-4 text-xs'>
            This demonstration uses a passwordless development identity. Enter any email address to
            continue. Production deployments sign in through the organisation&apos;s identity
            provider.
          </p>
        </div>
      </div>
    </main>
  );
};

export default LoginPage;
