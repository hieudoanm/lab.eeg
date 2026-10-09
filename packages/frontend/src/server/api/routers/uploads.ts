import { createUploadSession, finaliseUpload } from '@/lib/api';
import { revalidatePath } from 'next/cache';
import { z } from 'zod';

import { guard, organisationProcedure, router } from '../trpc';

const channelNames = z
  .array(z.string())
  .transform((names) => names.map((name) => name.trim()).filter((name) => name.length > 0))
  .pipe(z.array(z.string().min(1).max(100)).min(1).max(256));

export const uploadsRouter = router({
  createSession: organisationProcedure
    .input(
      z.object({
        datasetId: z.string().min(1),
        fileName: z.string().trim().min(1).max(255),
        fileSizeBytes: z.number().int().min(1),
        samplingFrequencyHz: z.number().positive(),
        channelNames,
        channelUnits: z.array(z.string()).nullish(),
      }),
    )
    .mutation(({ ctx, input }) => {
      return guard(() =>
        createUploadSession(
          ctx.membership.organisation_id,
          input.datasetId,
          {
            file_name: input.fileName,
            file_size_bytes: input.fileSizeBytes,
            manifest: {
              sampling_frequency_hz: input.samplingFrequencyHz,
              channel_names: input.channelNames,
              channel_units: input.channelUnits ?? null,
            },
          },
          ctx.client,
        ),
      );
    }),

  finalise: organisationProcedure
    .input(
      z.object({
        studyId: z.string().min(1),
        datasetId: z.string().min(1),
        sessionId: z.string().min(1),
      }),
    )
    .mutation(async ({ ctx, input }) => {
      try {
        return await guard(() =>
          finaliseUpload(
            ctx.membership.organisation_id,
            input.datasetId,
            input.sessionId,
            ctx.client,
          ),
        );
      } finally {
        revalidatePath(`/app/studies/${input.studyId}/datasets/${input.datasetId}`);
        revalidatePath(`/app/studies/${input.studyId}`);
      }
    }),
});

export type UploadsRouter = typeof uploadsRouter;
