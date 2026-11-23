'use client';

import { useEffect, useState } from 'react';

// site/R12 (handbook/guidelines/site-checklist.md): an edit survives a reload.
// A draft is kept per page and per starting text, so a changed starter or a
// different share link never brings back an edit made to something else.
const hash = (s: string) => {
  let h = 0x811c9dc5;
  for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 0x01000193);
  return (h >>> 0).toString(36);
};

export function useDraft(initial: string, scope = ''): [string, (s: string) => void] {
  const [value, setValue] = useState(initial);
  const [key, setKey] = useState<string | null>(null);
  useEffect(() => {
    const k = `cp-draft:${location.pathname}:${scope}:${hash(initial)}`;
    setKey(k);
    try {
      const saved = localStorage.getItem(k);
      setValue(saved ?? initial);
    } catch {
      setValue(initial);
    }
  }, [initial, scope]);
  const set = (s: string) => {
    setValue(s);
    if (!key) return;
    try {
      if (s === initial) localStorage.removeItem(key);
      else localStorage.setItem(key, s);
    } catch {
      // Storage full or blocked: the edit still works, it just won't survive a reload.
    }
  };
  return [value, set];
}
