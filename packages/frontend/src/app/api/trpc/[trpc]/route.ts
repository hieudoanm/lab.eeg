import { fetchRequestHandler } from '@trpc/server/adapters/fetch';

import { appRouter } from '@/server/api/root';
import { createTRPCContext } from '@/server/api/trpc';

/**
 * tRPC Backend-for-Frontend endpoint.
 *
 * The Python REST API is never called from the browser: the browser calls these
 * procedures, which run on the Next.js server, read the HttpOnly session cookie,
 * and forward authenticated requests upstream.
 */
const handler = (request: Request): Promise<Response> => {
  return fetchRequestHandler({
    endpoint: '/api/trpc',
    req: request,
    router: appRouter,
    createContext: () => createTRPCContext(),
  });
};

export { handler as GET, handler as POST };
