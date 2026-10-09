// EEG Lab Atoms - Basic building blocks

// Form atoms
export { Button } from './button';
export type { ButtonProps } from './button';

export { TextField } from './text-field';
export type { TextFieldProps } from './text-field';

export { TextareaField } from './textarea-field';
export type { TextareaFieldProps } from './textarea-field';

export { SelectField } from './select-field';
export type { SelectFieldProps, SelectOption } from './select-field';

// Display atoms
export { Card } from './card';
export type { CardProps } from './card';

export { DataTable } from './data-table';
export type { DataTableProps } from './data-table';

export { MetadataGrid } from './metadata-grid';
export type { MetadataGridProps, MetadataItem } from './metadata-grid';

export { StatusBadge } from './status-badge';
export type { StatusBadgeProps } from './status-badge';

export { ToastProvider, useToast } from './toast';
export type { ToastInput, ToastTone } from './toast';

// Theme
export {
  ThemeProvider,
  useTheme,
  themeBootScript,
  systemTheme,
  isLabTheme,
} from './theme-provider';
export type { LabTheme, ThemePreference, ThemeContextValue } from './theme-provider';

export { ThemeToggle } from './theme-toggle';

// State atoms
export { LoadingState, EmptyState, ErrorState, NoResultsState } from './states';
export type {
  LoadingStateProps,
  EmptyStateProps,
  ErrorStateProps,
  NoResultsStateProps,
} from './states';

// Navigation atoms
export { Breadcrumbs } from './breadcrumbs';
export type { Breadcrumb } from './breadcrumbs';

// Action atoms
export { SignOutButton } from './sign-out-button';

export { ArchiveStudyButton } from './archive-study-button';
export type { ArchiveStudyButtonProps } from './archive-study-button';

// Visualisation atoms
export { TimeSeriesChart } from './time-series-chart';
export type { TimeSeriesChartProps, TimeSeriesStats } from './time-series-chart';
export { computeTimeSeriesStats } from './time-series-chart';

export { StatisticsTable, StatisticsSummary } from './statistics-table';
export type { StatisticsTableProps, StatisticsSummaryProps } from './statistics-table';

export {
  AnalysisStatusBadge,
  AnalysisStatusDetail,
  analysisStatusIsTerminal,
  analysisStatusCanCancel,
} from './analysis-status';
export type { AnalysisStatusBadgeProps, AnalysisStatusDetailProps } from './analysis-status';
