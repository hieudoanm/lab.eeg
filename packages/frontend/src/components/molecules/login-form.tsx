'use client';

import { useRouter } from 'next/navigation';
import { useState, type FormEvent } from 'react';

import { Button } from '@/components/atoms/button';
import { TextField } from '@/components/atoms/text-field';
import { api } from '@/trpc/react';

interface ErrorPayload {
  data?: { httpStatus?: number };
}

const signInErrorMessage = (error: unknown): string => {
  const status = (error as ErrorPayload)?.data?.httpStatus;
  if (status === 404) {
    return 'Sign-in is not available in this environment.';
  }
  if (typeof status === 'number') {
    const message = error instanceof Error ? error.message : '';
    return message.length > 0
      ? message
      : 'We could not sign you in. Check your connection and try again.';
  }
  return 'We could not sign you in. Check your connection and try again.';
};

export const LoginForm = () => {
  const router = useRouter();
  const [email, setEmail] = useState('');
  const [error, setError] = useState<string | null>(null);
  const signIn = api.identity.signIn.useMutation();

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const value = email.trim();
    if (value.length === 0) {
      setError('Enter an email address to continue.');
      return;
    }

    setError(null);
    try {
      await signIn.mutateAsync({ subject: value, email: value });
    } catch (submitError) {
      setError(signInErrorMessage(submitError));
      return;
    }
    router.push('/app/dashboard');
    router.refresh();
  };

  return (
    <form className='mt-6 flex flex-col gap-4' onSubmit={handleSubmit} noValidate>
      <TextField
        name='email'
        type='email'
        label='Email address'
        autoComplete='email'
        placeholder='researcher@example.org'
        value={email}
        onChange={(event) => setEmail(event.target.value)}
        required
      />
      {error ? (
        <p role='alert' className='text-error text-sm'>
          {error}
        </p>
      ) : null}
      <Button type='submit' disabled={signIn.isPending}>
        {signIn.isPending ? 'Signing in…' : 'Sign in'}
      </Button>
    </form>
  );
};
