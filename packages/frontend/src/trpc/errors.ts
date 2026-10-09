import { TRPCError } from '@trpc/server';

import { ApiError } from '@/lib/api/errors';

export interface DescribedError {
  message: string;
  requestId: string | null;
}

/**
 * Describe a failure thrown by the BFF so server components can render an
 * {@link ErrorState} with a safe message and, where available, the upstream
 * request identifier.
 */
export const describeError = (error: unknown, fallback: string): DescribedError => {
  if (error instanceof TRPCError) {
    return {
      message: error.message,
      requestId: error.cause instanceof ApiError ? error.cause.requestId : null,
    };
  }
  return { message: fallback, requestId: null };
};
