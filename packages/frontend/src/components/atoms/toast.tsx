'use client';

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from 'react';

import { cn } from '@/lib/cn';

export type ToastTone = 'info' | 'success' | 'danger';

export interface ToastInput {
  title: string;
  description?: string;
  tone?: ToastTone;
}

interface Toast extends Required<Omit<ToastInput, 'description'>> {
  id: string;
  description?: string;
}

interface ToastContextValue {
  showToast: (input: ToastInput) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

const TONES: Record<ToastTone, string> = {
  info: 'alert-soft alert-info',
  success: 'alert-soft alert-success',
  danger: 'alert-soft alert-error',
};

let toastCounter = 0;

const nextId = (): string => {
  toastCounter += 1;
  const random = globalThis.crypto?.randomUUID?.();
  return random ?? `toast-${Date.now()}-${toastCounter}`;
};

export const ToastProvider = ({ children }: { children: ReactNode }) => {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const timers = useRef<Map<string, ReturnType<typeof setTimeout>>>(new Map());

  const dismiss = useCallback((id: string) => {
    setToasts((current) => current.filter((toast) => toast.id !== id));
    const timer = timers.current.get(id);
    if (timer) {
      clearTimeout(timer);
      timers.current.delete(id);
    }
  }, []);

  const showToast = useCallback(
    ({ title, description, tone = 'info' }: ToastInput) => {
      const id = nextId();
      setToasts((current) => [...current, { id, title, description, tone }]);
      const timer = setTimeout(() => dismiss(id), 6000);
      timers.current.set(id, timer);
    },
    [dismiss],
  );

  useEffect(() => {
    const activeTimers = timers.current;
    return () => {
      for (const timer of activeTimers.values()) {
        clearTimeout(timer);
      }
      activeTimers.clear();
    };
  }, []);

  const value = useMemo(() => ({ showToast }), [showToast]);

  return (
    <ToastContext.Provider value={value}>
      {children}
      <div aria-live='polite' aria-atomic='false' className='toast toast-end toast-bottom z-50'>
        {toasts.map((toast) => (
          <div
            key={toast.id}
            role='status'
            className={cn('alert pointer-events-auto shadow-sm', TONES[toast.tone])}>
            <div className='flex items-start justify-between gap-3'>
              <div>
                <p className='text-sm font-semibold'>{toast.title}</p>
                {toast.description ? (
                  <p className='mt-0.5 text-sm opacity-80'>{toast.description}</p>
                ) : null}
              </div>
              <button
                type='button'
                onClick={() => dismiss(toast.id)}
                className='text-sm opacity-60 hover:opacity-100'
                aria-label='Dismiss notification'>
                ×
              </button>
            </div>
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
};

export const useToast = (): ToastContextValue => {
  const context = useContext(ToastContext);
  if (context === null) {
    throw new Error('useToast must be used within a ToastProvider');
  }
  return context;
};
