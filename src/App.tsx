/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 */

import { useState } from 'react';
import { MotionConfig, motion } from 'motion/react';
import { ArrowDown, ExternalLink } from 'lucide-react';
import { CommandBox } from './components/CommandBox';
import { TuiSimulator } from './components/TuiSimulator';
import { CapabilitiesSection } from './components/CapabilitiesSection';
import {
  GITHUB_URL,
  Locale,
  RELEASES_URL,
  SITE_COPY,
} from './data/llmlabContent';

export default function App() {
  const [locale, setLocale] = useState<Locale>('ru');
  const copy = SITE_COPY[locale];

  const scrollToInstall = () => {
    const el = document.getElementById('install');
    el?.scrollIntoView({ behavior: 'smooth', block: 'center' });
  };

  return (
    <MotionConfig reducedMotion="user">
      <div className="min-h-screen bg-black text-white">
        {/* Top Bar Contract: 3 zones separated by gap-8 */}
        <header className="sticky top-0 z-40 border-b border-white/15 bg-black/95 backdrop-blur-sm">
          <div className="mx-auto flex max-w-6xl items-center justify-between gap-8 px-6 py-4">
            <a
              href="#"
              className="font-display text-xl font-bold tracking-tight text-white whitespace-nowrap shrink-0 focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-white"
            >
              llmLab
            </a>

            <nav
              aria-label="Primary navigation"
              className="hidden md:flex items-center gap-7 text-sm font-medium text-neutral-400"
            >
              <a
                href="#install"
                className="hover:text-white transition-colors whitespace-nowrap shrink-0"
              >
                {copy.nav.install}
              </a>
              <a
                href="#simulator"
                className="hover:text-white transition-colors whitespace-nowrap shrink-0"
              >
                {copy.nav.simulator}
              </a>
              <a
                href="#overview"
                className="hover:text-white transition-colors whitespace-nowrap shrink-0"
              >
                {copy.nav.overview}
              </a>
              <a
                href="#engines"
                className="hover:text-white transition-colors whitespace-nowrap shrink-0"
              >
                {copy.nav.engines}
              </a>
            </nav>

            <div className="shrink-0">
              <motion.button
                type="button"
                onClick={scrollToInstall}
                whileHover={{ scale: 1.03, y: -1 }}
                whileTap={{ scale: 0.95 }}
                transition={{ type: 'spring', stiffness: 440, damping: 24 }}
                className="inline-flex items-center gap-2 rounded-md bg-white px-4 py-2 text-xs font-semibold text-black hover:bg-neutral-200 whitespace-nowrap shrink-0 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white"
              >
                <span>{copy.nav.cta}</span>
                <ArrowDown className="h-3.5 w-3.5" aria-hidden="true" />
              </motion.button>
            </div>
          </div>
        </header>

        <main className="mx-auto max-w-6xl px-6 pb-24 pt-12 sm:pt-16 space-y-24">
          {/* Hero Section */}
          <section className="grid grid-cols-1 lg:grid-cols-12 gap-10 items-start">
            <div className="lg:col-span-6 space-y-6">
              <div className="flex flex-wrap items-center justify-between gap-4">
                <p className="font-mono text-xs text-neutral-400">
                  {copy.hero.meta}
                </p>

                <div
                  role="group"
                  aria-label="Language switch"
                  className="inline-flex items-center gap-1 rounded-md bg-[#0A0A0A] p-1 border border-white/15"
                >
                  {(['ru', 'en'] as const).map((lang) => {
                    const active = locale === lang;
                    return (
                      <motion.button
                        key={lang}
                        type="button"
                        onClick={() => setLocale(lang)}
                        whileHover={{ y: -1 }}
                        whileTap={{ scale: 0.95 }}
                        className={`relative px-2.5 py-1 text-xs font-semibold uppercase rounded-sm whitespace-nowrap shrink-0 transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white ${
                          active ? 'text-black' : 'text-neutral-400 hover:text-white'
                        }`}
                      >
                        {active && (
                          <motion.span
                            layoutId="locale-pill"
                            transition={{ type: 'spring', stiffness: 460, damping: 34 }}
                            className="absolute inset-0 rounded-sm bg-white"
                          />
                        )}
                        <span className="relative z-10">{lang}</span>
                      </motion.button>
                    );
                  })}
                </div>
              </div>

              <motion.h1
                key={`title-${locale}`}
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.25, ease: [0.16, 1, 0.3, 1] }}
                className="font-display font-bold tracking-tight text-white text-balance"
                style={{ fontSize: 'clamp(2.25rem, 3.8vw, 3.25rem)', lineHeight: 1.08 }}
              >
                {copy.hero.title}
              </motion.h1>

              <motion.p
                key={`sub-${locale}`}
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.28, delay: 0.04, ease: [0.16, 1, 0.3, 1] }}
                className="text-base leading-relaxed text-neutral-400 max-w-xl"
              >
                {copy.hero.subtitle}
              </motion.p>

              <CommandBox locale={locale} />
            </div>

            <div className="lg:col-span-6">
              <TuiSimulator locale={locale} />
            </div>
          </section>

          <CapabilitiesSection locale={locale} />
        </main>

        {/* Quiet Footer */}
        <footer className="border-t border-white/15 bg-black">
          <div className="mx-auto flex max-w-6xl flex-col sm:flex-row items-start sm:items-center justify-between gap-4 px-6 py-8 text-xs text-neutral-400">
            <p>{copy.footer.tagline}</p>

            <div className="flex items-center gap-6">
              <motion.a
                href={GITHUB_URL}
                target="_blank"
                rel="noreferrer"
                whileHover={{ y: -1, color: '#FFFFFF' }}
                whileTap={{ scale: 0.97 }}
                className="inline-flex items-center gap-1.5 hover:text-white transition-colors whitespace-nowrap shrink-0"
              >
                <span>{copy.footer.github}</span>
                <ExternalLink className="h-3.5 w-3.5" aria-hidden="true" />
              </motion.a>
              <motion.a
                href={RELEASES_URL}
                target="_blank"
                rel="noreferrer"
                whileHover={{ y: -1, color: '#FFFFFF' }}
                whileTap={{ scale: 0.97 }}
                className="inline-flex items-center gap-1.5 hover:text-white transition-colors whitespace-nowrap shrink-0"
              >
                <span>{copy.footer.releases}</span>
                <ExternalLink className="h-3.5 w-3.5" aria-hidden="true" />
              </motion.a>
            </div>
          </div>
        </footer>
      </div>
    </MotionConfig>
  );
}
