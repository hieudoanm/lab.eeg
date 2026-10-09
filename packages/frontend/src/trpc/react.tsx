'use client';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { httpBatchLink } from '@trpc/client';
import { createTRPCReact } from '@trpc/react-query';
import { useState, type ReactNode } from 'react';

import type { AppRouter } from '@/server/api/root';

/**
 * Typed tRPC React hooks for the browser.
 *
 * Requests go to the same-origin `/api/trpc` endpoint, so the HttpOnly session
 * cookie is sent automatically and never exposed to client JavaScript.
 */
export const api = createTRPCReact<AppRouter>();

const createQueryClient = (): QueryClient =>
  new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 30_000,
        refetchOnWindowFocus: false,
        retry: 1,
      },
    },
  });

export const TRPCReactProvider = ({ children }: { children: ReactNode }) => {
  const [queryClient] = useState(createQueryClient);
  const [trpcClient] = useState(() =>
    api.createClient({
      links: [
        httpBatchLink({
          url: '/api/trpc',
          fetch: (input, init) => fetch(input, { ...init, credentials: 'include' }),
        }),
      ],
    }),
  );

  return (
    <api.Provider client={trpcClient} queryClient={queryClient}>
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    </api.Provider>
  );
};
