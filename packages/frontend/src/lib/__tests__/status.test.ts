import { describe, expect, it } from 'vitest';

import {
  analysisJobStatusLabel,
  analysisJobStatusTone,
  datasetStatusLabel,
  datasetStatusTone,
  studyStatusLabel,
  studyStatusTone,
} from '../status';

describe('status', () => {
  it('returns study status tones', () => {
    expect(studyStatusTone('active')).toBe('success');
    expect(studyStatusTone('archived')).toBe('neutral');
  });

  it('returns study status labels', () => {
    expect(studyStatusLabel('active')).toBe('Active');
    expect(studyStatusLabel('archived')).toBe('Archived');
  });

  it('returns job status tones', () => {
    expect(analysisJobStatusTone('queued')).toBe('neutral');
    expect(analysisJobStatusTone('running')).toBe('info');
    expect(analysisJobStatusTone('succeeded')).toBe('success');
    expect(analysisJobStatusTone('failed')).toBe('danger');
    expect(analysisJobStatusTone('cancelled')).toBe('neutral');
  });

  it('returns job status labels', () => {
    expect(analysisJobStatusLabel('queued')).toBe('Queued');
    expect(analysisJobStatusLabel('running')).toBe('Running');
    expect(analysisJobStatusLabel('succeeded')).toBe('Succeeded');
    expect(analysisJobStatusLabel('failed')).toBe('Failed');
    expect(analysisJobStatusLabel('cancelled')).toBe('Cancelled');
  });

  it('returns dataset status tones', () => {
    expect(datasetStatusTone('pending_upload')).toBe('neutral');
    expect(datasetStatusTone('validating')).toBe('info');
    expect(datasetStatusTone('ready')).toBe('success');
    expect(datasetStatusTone('rejected')).toBe('danger');
    expect(datasetStatusTone('failed')).toBe('danger');
    expect(datasetStatusTone('archived')).toBe('neutral');
  });

  it('returns dataset status labels', () => {
    expect(datasetStatusLabel('pending_upload')).toBe('Pending upload');
    expect(datasetStatusLabel('validating')).toBe('Validating');
    expect(datasetStatusLabel('ready')).toBe('Ready');
    expect(datasetStatusLabel('rejected')).toBe('Rejected');
    expect(datasetStatusLabel('failed')).toBe('Failed');
    expect(datasetStatusLabel('archived')).toBe('Archived');
  });
});
