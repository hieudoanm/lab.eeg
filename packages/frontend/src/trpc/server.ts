import 'server-only';

import { TRPCError } from '@trpc/server';
import { redirect } from 'next/navigation';

import { appRouter } from '@/server/api/root';
import { createTRPCContext } from '@/server/api/trpc';

/**
 * Build a tRPC caller for the current request.
 *
 * React Server Components use this to read data directly from the BFF router
 * without an HTTP round trip, while sharing the same procedures (and therefore
 * the same types and authorisation rules) used by client components.
 */
export const getCaller = async () => {
  const context = await createTRPCContext();
  return appRouter.createCaller(context);
};

export type Caller = Awaited<ReturnType<typeof getCaller>>;

/**
 * Resolve the current session for a server component, redirecting anonymous
 * visitors to the sign-in page.
 */
export const requireSession = async () => {
  const caller = await getCaller();
  try {
    const session = await caller.identity.session();
    return { caller, ...session };
  } catch (error) {
    if (error instanceof TRPCError && error.code === 'UNAUTHORIZED') {
      redirect('/login');
    }
    throw error;
  }
};
