import {
  cancelAnalysis,
  fetchAnalyses,
  fetchOrganisationAnalyses,
  fetchAnalysisJob,
  fetchAnalysisResult,
  submitAnalysis,
} from '@/lib/api';
import { z } from 'zod';

import { guard, organisationProcedure, router } from '../trpc';

const parameters = z.object({
  include_mean: z.boolean().optional(),
  include_standard_deviation: z.boolean().optional(),
  include_min_max: z.boolean().optional(),
  include_peak_to_peak: z.boolean().optional(),
});

export const analysesRouter = router({
  list: organisationProcedure
    .input(
      z.object({
        datasetId: z.string().min(1),
        cursor: z.string().min(1).optional(),
        limit: z.number().int().positive().max(100).optional(),
      }),
    )
    .query(({ ctx, input }) => {
      return guard(() =>
        fetchAnalyses(
          ctx.membership.organisation_id,
          input.datasetId,
          { cursor: input.cursor, limit: input.limit },
          ctx.client,
        ),
      );
    }),

  listOrganisation: organisationProcedure
    .input(
      z.object({
        cursor: z.string().min(1).optional(),
        limit: z.number().int().positive().max(100).optional(),
        datasetId: z.string().min(1).optional(),
        studyId: z.string().min(1).optional(),
        status: z.enum(['queued', 'running', 'succeeded', 'failed', 'cancelled']).optional(),
      }),
    )
    .query(({ ctx, input }) => {
      return guard(() =>
        fetchOrganisationAnalyses(
          ctx.membership.organisation_id,
          {
            cursor: input.cursor,
            limit: input.limit,
            datasetId: input.datasetId,
            studyId: input.studyId,
            status: input.status,
          },
          ctx.client,
        ),
      );
    }),

  byId: organisationProcedure
    .input(z.object({ jobId: z.string().min(1) }))
    .query(({ ctx, input }) => {
      return guard(() => fetchAnalysisJob(ctx.membership.organisation_id, input.jobId, ctx.client));
    }),

  result: organisationProcedure
    .input(z.object({ jobId: z.string().min(1) }))
    .query(({ ctx, input }) => {
      return guard(() =>
        fetchAnalysisResult(ctx.membership.organisation_id, input.jobId, ctx.client),
      );
    }),

  submit: organisationProcedure
    .input(
      z.object({
        datasetId: z.string().min(1),
        algorithmVersion: z.string().min(1),
        parameters,
        idempotencyKey: z.string().min(1),
      }),
    )
    .mutation(({ ctx, input }) => {
      return guard(() =>
        submitAnalysis(
          ctx.membership.organisation_id,
          input.datasetId,
          {
            analysis_type: 'descriptive_statistics',
            algorithm_version: input.algorithmVersion,
            parameters: input.parameters,
          },
          input.idempotencyKey,
          ctx.client,
        ),
      );
    }),

  cancel: organisationProcedure
    .input(z.object({ jobId: z.string().min(1) }))
    .mutation(({ ctx, input }) => {
      return guard(() => cancelAnalysis(ctx.membership.organisation_id, input.jobId, ctx.client));
    }),
});

export type AnalysesRouter = typeof analysesRouter;
