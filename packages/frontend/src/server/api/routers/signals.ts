import { fetchSignalWindow } from '@/lib/api';
import { z } from 'zod';

import { guard, organisationProcedure, router } from '../trpc';

export const signalsRouter = router({
  window: organisationProcedure
    .input(
      z.object({
        datasetId: z.string().min(1),
        startSeconds: z.number().min(0),
        endSeconds: z.number().positive(),
        channels: z.array(z.number().int().nonnegative()).optional(),
      }),
    )
    .query(({ ctx, input }) => {
      return guard(() =>
        fetchSignalWindow(
          ctx.membership.organisation_id,
          input.datasetId,
          {
            startSeconds: input.startSeconds,
            endSeconds: input.endSeconds,
            channels: input.channels,
          },
          ctx.client,
        ),
      );
    }),
});

export type SignalsRouter = typeof signalsRouter;
