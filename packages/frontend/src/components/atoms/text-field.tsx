import type { InputHTMLAttributes } from 'react';

import { cn } from '@/lib/cn';

export interface TextFieldProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'id'> {
  id?: string;
  label: string;
  hint?: string;
  error?: string;
}

export const TextField = ({
  id,
  label,
  hint,
  error,
  name,
  required,
  className,
  ...props
}: TextFieldProps) => {
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
      <input
        id={fieldId}
        name={name}
        required={required}
        aria-invalid={error ? true : undefined}
        aria-describedby={describedBy}
        className={cn(
          'input w-full disabled:cursor-not-allowed disabled:opacity-60',
          error ? 'input-error' : '',
          className,
        )}
        {...props}
      />
      {error ? (
        <p id={errorId} role='alert' className='text-error text-xs'>
          {error}
        </p>
      ) : null}
    </div>
  );
};
