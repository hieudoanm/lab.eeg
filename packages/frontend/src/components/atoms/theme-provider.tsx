'use client';

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useSyncExternalStore,
  type ReactNode,
} from 'react';

/**
 * Lab dual-theme provider.
 *
 * The Lab design system ships two themes — `lab-light` and `lab-dark`
 * (see globals.css). The provider resolves the active theme, applies it as a
 * `data-theme` attribute on the root element, and persists an explicit user
 * choice in localStorage (`lab-theme`). When no choice is stored the theme
 * follows the operating system `prefers-color-scheme` setting.
 *
 * The active theme is read from an external store (localStorage + matchMedia)
 * via `useSyncExternalStore`, so system changes and choices made in other tabs
 * propagate without synchronising state inside effects.
 */

export const THEME_STORAGE_KEY = 'lab-theme';

export type LabTheme = 'lab-light' | 'lab-dark';

export type ThemePreference = LabTheme | null;

export const isLabTheme = (value: string | null): value is LabTheme =>
  value === 'lab-light' || value === 'lab-dark';

export const systemTheme = (): LabTheme =>
  typeof window !== 'undefined' &&
  typeof window.matchMedia === 'function' &&
  window.matchMedia('(prefers-color-scheme: dark)').matches
    ? 'lab-dark'
    : 'lab-light';

const listeners = new Set<() => void>();

const emit = (): void => {
  for (const listener of listeners) listener();
};

const subscribe = (onStoreChange: () => void): (() => void) => {
  if (typeof window === 'undefined') return () => undefined;
  const mediaQuery =
    typeof window.matchMedia === 'function'
      ? window.matchMedia('(prefers-color-scheme: dark)')
      : null;
  mediaQuery?.addEventListener('change', onStoreChange);
  window.addEventListener('storage', onStoreChange);
  listeners.add(onStoreChange);
  return () => {
    mediaQuery?.removeEventListener('change', onStoreChange);
    window.removeEventListener('storage', onStoreChange);
    listeners.delete(onStoreChange);
  };
};

const readPreference = (): ThemePreference => {
  if (typeof window === 'undefined') return null;
  const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
  return isLabTheme(stored) ? stored : null;
};

const getThemeSnapshot = (): LabTheme => readPreference() ?? systemTheme();

const getPreferenceSnapshot = (): ThemePreference => readPreference();

const getServerThemeSnapshot = (): LabTheme => 'lab-light';

const getServerPreferenceSnapshot = (): ThemePreference => null;

export interface ThemeContextValue {
  /** The theme currently applied to the document. */
  theme: LabTheme;
  /** The stored explicit choice, or `null` when following the system. */
  preference: ThemePreference;
  /** Persist and apply an explicit theme choice. */
  setTheme: (theme: LabTheme) => void;
  /** Clear the stored choice and follow the operating-system preference again. */
  resetTheme: () => void;
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

const fallbackContext: ThemeContextValue = {
  theme: 'lab-light',
  preference: null,
  setTheme: () => undefined,
  resetTheme: () => undefined,
};

export const ThemeProvider = ({ children }: { children: ReactNode }) => {
  const theme = useSyncExternalStore(subscribe, getThemeSnapshot, getServerThemeSnapshot);
  const preference = useSyncExternalStore(
    subscribe,
    getPreferenceSnapshot,
    getServerPreferenceSnapshot,
  );

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  const setTheme = useCallback((next: LabTheme) => {
    window.localStorage.setItem(THEME_STORAGE_KEY, next);
    emit();
  }, []);

  const resetTheme = useCallback(() => {
    window.localStorage.removeItem(THEME_STORAGE_KEY);
    emit();
  }, []);

  const value = useMemo(
    () => ({ theme, preference, setTheme, resetTheme }),
    [preference, resetTheme, setTheme, theme],
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
};

/** Read the active theme. Falls back to the light theme when rendered outside a provider. */
export const useTheme = (): ThemeContextValue => {
  return useContext(ThemeContext) ?? fallbackContext;
};

/**
 * Script-safe boot helper used by the layout to set `data-theme` before the
 * app hydrates, avoiding a flash of the wrong palette.
 */
export const themeBootScript = `
  (function () {
    try {
      var stored = window.localStorage.getItem('${THEME_STORAGE_KEY}');
      var theme = (stored === 'lab-light' || stored === 'lab-dark')
        ? stored
        : (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'lab-dark' : 'lab-light');
      document.documentElement.dataset.theme = theme;
    } catch (error) { /* no-op */ }
  })();
`;
