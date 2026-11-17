import { useState } from 'react';
import { AnimatePresence, motion } from 'motion/react';
import { ENGINE_SPECS, EngineId, Locale, SITE_COPY } from '../data/llmlabContent';

interface CapabilitiesSectionProps {
  locale: Locale;
}

export function CapabilitiesSection({ locale }: CapabilitiesSectionProps) {
  const [selectedEngine, setSelectedEngine] = useState<EngineId>('avx2');
  const copy = SITE_COPY[locale];

  const activeEngine =
    ENGINE_SPECS.find((e) => e.id === selectedEngine) ?? ENGINE_SPECS[0];

  return (
    <div className="space-y-24">
      {/* Capabilities Asymmetric Grid */}
      <section id="overview" className="scroll-mt-24">
        <div className="max-w-2xl">
          <h2 className="font-display text-2xl sm:text-3xl font-semibold tracking-tight text-white">
            {copy.capabilities.heading}
          </h2>
          <p className="mt-3 text-base leading-relaxed text-slate-400">
            {copy.capabilities.lead}
          </p>
        </div>

        <div className="mt-10 grid grid-cols-1 md:grid-cols-3 gap-5">
          {copy.capabilities.items.map((item, idx) => {
            const wide = idx === 0 || idx === 3;
            return (
              <motion.article
                key={item.num}
                initial={{ opacity: 0, y: 14 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true, amount: 0.2 }}
                transition={{
                  duration: 0.3,
                  delay: idx * 0.05,
                  ease: [0.16, 1, 0.3, 1],
                }}
                whileHover={{ y: -3 }}
                className={`rounded-xl border border-white/10 bg-[#0D1219] p-6 transition-colors hover:border-[#00F5D4]/35 flex flex-col justify-between gap-6 ${
                  wide ? 'md:col-span-2' : 'md:col-span-1'
                }`}
              >
                <div>
                  <span className="font-mono text-xs font-medium text-[#00F5D4]">
                    {item.num}
                  </span>
                  <h3 className="mt-2 font-display text-lg font-semibold text-white">
                    {item.title}
                  </h3>
                  <p className="mt-2.5 text-sm leading-relaxed text-slate-300">
                    {item.detail}
                  </p>
                </div>

                <p className="font-mono text-xs text-slate-400 border-t border-white/10 pt-3">
                  {item.metric}
                </p>
              </motion.article>
            );
          })}
        </div>
      </section>

      {/* Engine Profiles Selector */}
      <section id="engines" className="scroll-mt-24">
        <div className="max-w-2xl">
          <h2 className="font-display text-2xl sm:text-3xl font-semibold tracking-tight text-white">
            {copy.engines.heading}
          </h2>
          <p className="mt-3 text-base leading-relaxed text-slate-400">
            {copy.engines.lead}
          </p>
        </div>

        <div className="mt-8 rounded-xl border border-white/10 bg-[#0D1219] p-5 sm:p-6">
          <div
            role="tablist"
            aria-label="Inference engine profiles"
            className="flex flex-wrap gap-2 border-b border-white/10 pb-4"
          >
            {ENGINE_SPECS.map((engine) => {
              const active = engine.id === selectedEngine;
              return (
                <motion.button
                  key={engine.id}
                  type="button"
                  role="tab"
                  aria-selected={active}
                  onClick={() => setSelectedEngine(engine.id)}
                  whileHover={{ y: -1 }}
                  whileTap={{ scale: 0.97 }}
                  transition={{ duration: 0.14 }}
                  className={`relative rounded-lg px-4 py-2 text-xs sm:text-sm font-medium whitespace-nowrap shrink-0 transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#00F5D4] ${
                    active ? 'text-[#07090D]' : 'text-slate-300 hover:text-white bg-[#07090D]'
                  }`}
                >
                  {active && (
                    <motion.span
                      layoutId="engine-tab-pill"
                      transition={{ type: 'spring', stiffness: 450, damping: 32 }}
                      className="absolute inset-0 rounded-lg bg-[#00F5D4]"
                    />
                  )}
                  <span className="relative z-10">{engine.label}</span>
                </motion.button>
              );
            })}
          </div>

          <AnimatePresence mode="wait">
            <motion.div
              key={activeEngine.id}
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ duration: 0.18, ease: [0.16, 1, 0.3, 1] }}
              className="mt-6 grid grid-cols-1 lg:grid-cols-12 gap-6 items-start"
            >
              <div className="lg:col-span-7 space-y-3">
                <div className="flex items-center gap-3 text-xs font-mono text-[#00F5D4]">
                  <span>{activeEngine.label}</span>
                  <span aria-hidden="true">·</span>
                  <span className="text-slate-400">{activeEngine.target}</span>
                </div>
                <p className="text-sm sm:text-base leading-relaxed text-slate-200">
                  {activeEngine.summary[locale]}
                </p>
                <p className="text-xs sm:text-sm leading-relaxed text-slate-400">
                  {activeEngine.speedNote[locale]}
                </p>
              </div>

              <div className="lg:col-span-5 rounded-lg border border-white/10 bg-[#07090D] p-4">
                <p className="text-xs font-medium text-slate-400">
                  {copy.engines.flagsLabel}
                </p>
                <code className="mt-2 block font-mono text-xs leading-relaxed text-[#00F5D4] break-words">
                  {activeEngine.flags}
                </code>
              </div>
            </motion.div>
          </AnimatePresence>
        </div>
      </section>

      {/* 3-Step Quick Start Tutorial */}
      <section id="tutorial" className="scroll-mt-24">
        <h2 className="font-display text-2xl sm:text-3xl font-semibold tracking-tight text-white">
          {copy.tutorial.heading}
        </h2>

        <div className="mt-8 grid grid-cols-1 md:grid-cols-3 gap-5">
          {copy.tutorial.steps.map((s, idx) => (
            <motion.div
              key={s.step}
              initial={{ opacity: 0, y: 14 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, amount: 0.2 }}
              transition={{
                duration: 0.3,
                delay: idx * 0.06,
                ease: [0.16, 1, 0.3, 1],
              }}
              whileHover={{ y: -2 }}
              className="rounded-xl border border-white/10 bg-[#0D1219] p-6"
            >
              <span className="font-mono text-xs font-semibold text-[#3A86FF]">
                {s.step}
              </span>
              <h3 className="mt-2 font-display text-lg font-semibold text-white">
                {s.title}
              </h3>
              <p className="mt-2.5 text-sm leading-relaxed text-slate-300">
                {s.body}
              </p>
            </motion.div>
          ))}
        </div>
      </section>
    </div>
  );
}
