// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

import { LOCALE_CATALOG } from './locales/catalog.js';

export const AVAILABLE_LANGUAGES = [
  { code: 'en', name: 'English', nativeName: 'English' },
  { code: 'es', name: 'Spanish', nativeName: 'Español' },
  { code: 'fr', name: 'French', nativeName: 'Français' },
  { code: 'de', name: 'German', nativeName: 'Deutsch' },
  { code: 'ja', name: 'Japanese', nativeName: '日本語' },
  { code: 'zh-Hans', name: 'Chinese (Simplified)', nativeName: '简体中文' },
  { code: 'zh-Hant', name: 'Chinese (Traditional)', nativeName: '繁體中文' },
  { code: 'ar', name: 'Arabic', nativeName: 'العربية' },
  { code: 'bn', name: 'Bengali', nativeName: 'বাংলা' },
  { code: 'cs', name: 'Czech', nativeName: 'Čeština' },
  { code: 'da', name: 'Danish', nativeName: 'Dansk' },
  { code: 'fi', name: 'Finnish', nativeName: 'Suomi' },
  { code: 'he', name: 'Hebrew', nativeName: 'עברית' },
  { code: 'hi', name: 'Hindi', nativeName: 'हिन्दी' },
  { code: 'id', name: 'Indonesian', nativeName: 'Bahasa Indonesia' },
  { code: 'it', name: 'Italian', nativeName: 'Italiano' },
  { code: 'ko', name: 'Korean', nativeName: '한국어' },
  { code: 'nb', name: 'Norwegian', nativeName: 'Norsk Bokmål' },
  { code: 'nl', name: 'Dutch', nativeName: 'Nederlands' },
  { code: 'pl', name: 'Polish', nativeName: 'Polski' },
  { code: 'pt-BR', name: 'Portuguese (Brazil)', nativeName: 'Português (Brasil)' },
  { code: 'pt-PT', name: 'Portuguese (Portugal)', nativeName: 'Português (Portugal)' },
  { code: 'ru', name: 'Russian', nativeName: 'Русский' },
  { code: 'sv', name: 'Swedish', nativeName: 'Svenska' },
  { code: 'th', name: 'Thai', nativeName: 'ไทย' },
  { code: 'tr', name: 'Turkish', nativeName: 'Türkçe' },
  { code: 'uk', name: 'Ukrainian', nativeName: 'Українська' },
  { code: 'vi', name: 'Vietnamese', nativeName: 'Tiếng Việt' },
];

let currentLocale = 'en';

export function initLanguage(initialLang) {
  const saved = initialLang || localStorage.getItem('clawvinci_language') || 'en';
  setLanguage(saved);
}

export function setLanguage(langCode) {
  if (LOCALE_CATALOG[langCode]) {
    currentLocale = langCode;
  } else {
    currentLocale = 'en';
  }
  localStorage.setItem('clawvinci_language', currentLocale);
  updateDomTranslations();
  window.dispatchEvent(new CustomEvent('clawvinci-locale-changed', { detail: { locale: currentLocale } }));
}

export function getLanguage() {
  return currentLocale;
}

export function t(key, fallback) {
  const table = LOCALE_CATALOG[currentLocale] || LOCALE_CATALOG['en'] || {};
  if (table[key] !== undefined) {
    return table[key];
  }
  if (currentLocale !== 'en' && LOCALE_CATALOG['en'] && LOCALE_CATALOG['en'][key] !== undefined) {
    return LOCALE_CATALOG['en'][key];
  }
  return fallback !== undefined ? fallback : key;
}

export function updateDomTranslations() {
  document.querySelectorAll('[data-i18n]').forEach((el) => {
    const key = el.getAttribute('data-i18n');
    if (key) {
      el.textContent = t(key);
    }
  });
  document.querySelectorAll('[data-i18n-title]').forEach((el) => {
    const key = el.getAttribute('data-i18n-title');
    if (key) {
      el.setAttribute('title', t(key));
    }
  });
  document.querySelectorAll('[data-i18n-placeholder]').forEach((el) => {
    const key = el.getAttribute('data-i18n-placeholder');
    if (key) {
      el.setAttribute('placeholder', t(key));
    }
  });
}
