import { fetchDataset, fetchDatasets, registerDataset } from '@/lib/api';
import { revalidatePath } from 'next/cache';
import { z } from 'zod';

import { guard, organisationProcedure, router } from '../trpc';

export const datasetsRouter = router({
  list: organisationProcedure
    .input(
      z.object({
        studyId: z.string().min(1),
        cursor: z.string().min(1).optional(),
        limit: z.number().int().positive().max(100).optional(),
      }),
    )
    .query(({ ctx, input }) => {
      return guard(() =>
        fetchDatasets(
          ctx.membership.organisation_id,
          input.studyId,
          { cursor: input.cursor, limit: input.limit },
          ctx.client,
        ),
      );
    }),

  byId: organisationProcedure
    .input(z.object({ datasetId: z.string().min(1) }))
    .query(({ ctx, input }) => {
      return guard(() => fetchDataset(ctx.membership.organisation_id, input.datasetId, ctx.client));
    }),

  register: organisationProcedure
    .input(
      z.object({
        studyId: z.string().min(1),
        name: z.string().trim().min(1).max(200),
        description: z.string().trim().max(2000).nullish(),
        sourceDescription: z.string().trim().max(2000).nullish(),
      }),
    )
    .mutation(async ({ ctx, input }) => {
      const dataset = await guard(() =>
        registerDataset(
          ctx.membership.organisation_id,
          input.studyId,
          {
            name: input.name,
            description: input.description ?? null,
            source_description: input.sourceDescription ?? null,
          },
          ctx.client,
        ),
      );
      revalidatePath(`/app/studies/${input.studyId}`);
      return dataset;
    }),
});

export type DatasetsRouter = typeof datasetsRouter;
