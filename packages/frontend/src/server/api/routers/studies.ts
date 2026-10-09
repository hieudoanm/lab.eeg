import { archiveStudy, createStudy, fetchStudies, fetchStudy, updateStudy } from '@/lib/api';
import { revalidatePath } from 'next/cache';
import { z } from 'zod';

import { guard, organisationProcedure, router } from '../trpc';

const studyFields = {
  name: z.string().trim().min(1).max(200),
  description: z.string().trim().max(2000).nullish(),
};

export const studiesRouter = router({
  list: organisationProcedure
    .input(
      z.object({
        search: z.string().min(1).optional(),
        status: z.enum(['active', 'archived']).optional(),
        cursor: z.string().min(1).optional(),
        limit: z.number().int().positive().max(100).optional(),
      }),
    )
    .query(({ ctx, input }) => {
      return guard(() =>
        fetchStudies(
          ctx.membership.organisation_id,
          { search: input.search, status: input.status, cursor: input.cursor, limit: input.limit },
          ctx.client,
        ),
      );
    }),

  byId: organisationProcedure
    .input(z.object({ studyId: z.string().min(1) }))
    .query(({ ctx, input }) => {
      return guard(() => fetchStudy(ctx.membership.organisation_id, input.studyId, ctx.client));
    }),

  create: organisationProcedure.input(z.object(studyFields)).mutation(async ({ ctx, input }) => {
    const study = await guard(() =>
      createStudy(
        ctx.membership.organisation_id,
        { name: input.name, description: input.description ?? null },
        ctx.client,
      ),
    );
    revalidatePath('/app/studies');
    return study;
  }),

  update: organisationProcedure
    .input(z.object({ studyId: z.string().min(1), ...studyFields }))
    .mutation(async ({ ctx, input }) => {
      const study = await guard(() =>
        updateStudy(
          ctx.membership.organisation_id,
          input.studyId,
          { name: input.name, description: input.description ?? null },
          ctx.client,
        ),
      );
      revalidatePath('/app/studies');
      revalidatePath(`/app/studies/${input.studyId}`);
      return study;
    }),

  archive: organisationProcedure
    .input(z.object({ studyId: z.string().min(1) }))
    .mutation(async ({ ctx, input }) => {
      const result = await guard(() =>
        archiveStudy(ctx.membership.organisation_id, input.studyId, ctx.client),
      );
      revalidatePath('/app/studies');
      revalidatePath(`/app/studies/${input.studyId}`);
      return result;
    }),
});

export type StudiesRouter = typeof studiesRouter;
