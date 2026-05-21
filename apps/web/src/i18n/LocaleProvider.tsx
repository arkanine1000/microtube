import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react';
import { COPY, LANGUAGE_OPTIONS, LOCALES, type Copy, type Locale } from './copy';

const STORAGE_KEY = 'microtube.locale';

interface LocaleContextValue {
  locale: Locale;
  copy: Copy;
  languages: typeof LANGUAGE_OPTIONS;
  setLocale: (locale: Locale) => void;
}

const LocaleContext = createContext<LocaleContextValue | null>(null);

function isLocale(value: string | null | undefined): value is Locale {
  return LOCALES.includes(value as Locale);
}

function readSavedLocale(): Locale | null {
  try {
    return isLocale(window.localStorage.getItem(STORAGE_KEY))
      ? (window.localStorage.getItem(STORAGE_KEY) as Locale)
      : null;
  } catch {
    return null;
  }
}

function browserLocale(): Locale {
  const languages = navigator.languages.length
    ? navigator.languages
    : [navigator.language];
  return languages.some((language) => language.toLowerCase().startsWith('hr'))
    ? 'hr'
    : 'en';
}

export function getInitialLocale(): Locale {
  if (typeof window === 'undefined' || typeof navigator === 'undefined') {
    return 'en';
  }
  return readSavedLocale() ?? browserLocale();
}

export function LocaleProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(getInitialLocale);

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  const setLocale = useCallback((nextLocale: Locale) => {
    setLocaleState(nextLocale);
    try {
      window.localStorage.setItem(STORAGE_KEY, nextLocale);
    } catch {
      // The app still switches language if storage is unavailable.
    }
  }, []);

  const value = useMemo<LocaleContextValue>(
    () => ({
      locale,
      copy: COPY[locale],
      languages: LANGUAGE_OPTIONS,
      setLocale,
    }),
    [locale, setLocale],
  );

  return (
    <LocaleContext.Provider value={value}>{children}</LocaleContext.Provider>
  );
}

export function useLocale() {
  const value = useContext(LocaleContext);
  if (!value) {
    throw new Error('useLocale must be used inside LocaleProvider');
  }
  return value;
}
