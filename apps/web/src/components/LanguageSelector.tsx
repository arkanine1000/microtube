import { Check, ChevronDown } from 'lucide-react';
import { useState } from 'react';
import { useLocale } from '../i18n/LocaleProvider';

export function LanguageSelector() {
  const { copy, languages, locale, setLocale } = useLocale();
  const [open, setOpen] = useState(false);
  const active = languages.find((language) => language.locale === locale);

  return (
    <div
      className="language-select"
      onBlur={(event) => {
        const nextFocus = event.relatedTarget as Node | null;
        if (!nextFocus || !event.currentTarget.contains(nextFocus)) {
          setOpen(false);
        }
      }}
      onKeyDown={(event) => {
        if (event.key === 'Escape') {
          setOpen(false);
        }
      }}
    >
      <button
        className="language-toggle"
        type="button"
        aria-label={copy.language.toggleLabel}
        title={copy.language.selectorLabel}
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen((value) => !value)}
      >
        <span aria-hidden="true">{active?.flag}</span>
        <ChevronDown size={16} strokeWidth={2.4} />
      </button>

      {open && (
        <div
          className="language-menu"
          role="menu"
          aria-label={copy.language.menuLabel}
        >
          {languages.map((language) => {
            const selected = language.locale === locale;
            return (
              <button
                key={language.locale}
                className={`language-option${selected ? ' active' : ''}`}
                type="button"
                role="menuitemradio"
                aria-checked={selected}
                onClick={() => {
                  setLocale(language.locale);
                  setOpen(false);
                }}
              >
                <span className="language-flag" aria-hidden="true">
                  {language.flag}
                </span>
                <span>{language.name}</span>
                {selected && <Check size={15} strokeWidth={2.5} />}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
