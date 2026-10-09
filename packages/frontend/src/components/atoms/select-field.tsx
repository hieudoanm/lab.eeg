import type { SelectHTMLAttributes } from 'react';

import { cn } from '@/lib/cn';

export interface SelectOption {
  value: string;
  label: string;
}

export interface SelectFieldProps extends Omit<SelectHTMLAttributes<HTMLSelectElement>, 'id'> {
  id?: string;
  label: string;
  options: SelectOption[];
  hint?: string;
  error?: string;
}

export const SelectField = ({
  id,
  label,
  options,
  hint,
  error,
  name,
  required,
  className,
  ...props
}: SelectFieldProps) => {
  const fieldId = id ?? name ?? label.toLowerCase().replace(/\s+/g, '-');
  const hintId = hint ? `${fieldId}-hint` : undefined;
  const errorId = error ? `${fieldId}-error` : undefined;
  const describedBy = [hintId, errorId].filter(Boolean).join(' ') || undefined;

  return (
    <div className='flex flex-col gap-1.5'>
      <label htmlFor={fieldId} className='text-base-content text-sm font-medium'>
        {label}
        {required ? (
          <span aria-hidden='true' className='text-error ml-1'>
            *
          </span>
        ) : null}
      </label>
      {hint ? (
        <p id={hintId} className='text-base-content/60 text-xs'>
          {hint}
        </p>
      ) : null}
      <select
        id={fieldId}
        name={name}
        required={required}
        aria-invalid={error ? true : undefined}
        aria-describedby={describedBy}
        className={cn(
          'select w-full disabled:cursor-not-allowed disabled:opacity-60',
          error ? 'select-error' : '',
          className,
        )}
        {...props}>
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
      {error ? (
        <p id={errorId} role='alert' className='text-error text-xs'>
          {error}
        </p>
      ) : null}
    </div>
  );
};
