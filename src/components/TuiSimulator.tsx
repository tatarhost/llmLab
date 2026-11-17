import { useEffect, useState } from 'react';
import { AnimatePresence, motion } from 'motion/react';
import { Play, RotateCcw, ShieldCheck, Sparkles } from 'lucide-react';
import {
  calculateFreedRamMb,
  formatRamGb,
  INITIAL_FOCUS_PROCESSES,
  Locale,
  LOGO_LINES,
  ProcessSample,
  SITE_COPY,
} from '../data/llmlabContent';

type TuiTab = 'splash' | 'focus' | 'hud';

interface TuiSimulatorProps {
  locale: Locale;
}

const SAMPLE_RESPONSE: Record<Locale, string> = {
  ru: 'План питания переключён на High Performance. Потоки привязаны к 6 быстрым P-ядрам, KV-кэш квантован в q8_0. Скорость генерации на CPU стабильно держится на уровне 28.4 tok/s без троттлинга.',
  en: 'Power plan switched to High Performance. Threads pinned to 6 fast P-cores with q8_0 KV cache quantization. CPU generation holds steady at 28.4 tok/s without thermal throttling.',
};

export function TuiSimulator({ locale }: TuiSimulatorProps) {
  const [activeTab, setActiveTab] = useState<TuiTab>('splash');
  const [scanRow, setScanRow] = useState<number>(LOGO_LINES.length);
  const [pulseActive, setPulseActive] = useState(false);

  const [processes, setProcesses] = useState<ProcessSample[]>(INITIAL_FOCUS_PROCESSES);
  const [cleanedIds, setCleanedIds] = useState<Set<string>>(new Set());
  const [freedTotalMb, setFreedTotalMb] = useState(0);

  const [streamChars, setStreamChars] = useState(SAMPLE_RESPONSE[locale].length);
  const [isStreaming, setIsStreaming] = useState(false);

  const copy = SITE_COPY[locale].simulator;

  // Replay 60 FPS splash scanline effect
  const replayScan = () => {
    setScanRow(0);
    setPulseActive(false);
  };

  useEffect(() => {
    if (activeTab !== 'splash' || scanRow >= LOGO_LINES.length) return;
    const timer = window.setTimeout(() => {
      const next = scanRow + 1;
      setScanRow(next);
      if (next === LOGO_LINES.length) {
        setPulseActive(true);
        window.setTimeout(() => setPulseActive(false), 380);
      }
    }, 34);
    return () => window.clearTimeout(timer);
  }, [activeTab, scanRow]);

  // Reset streamed text cleanly when locale changes
  useEffect(() => {
    if (!isStreaming) {
      setStreamChars(SAMPLE_RESPONSE[locale].length);
    }
  }, [locale, isStreaming]);

  useEffect(() => {
    if (!isStreaming) return;
    const fullText = SAMPLE_RESPONSE[locale];
    if (streamChars >= fullText.length) {
      setIsStreaming(false);
      return;
    }
    const timer = window.setTimeout(() => {
      setStreamChars((prev) => Math.min(prev + 4, fullText.length));
    }, 24);
    return () => window.clearTimeout(timer);
  }, [isStreaming, streamChars, locale]);

  const toggleKeep = (id: string) => {
    setProcesses((prev) =>
      prev.map((p) => (p.id === id ? { ...p, keep: !p.keep } : p))
    );
  };

  const handleApplyCleanup = () => {
    const toClean = processes.filter((p) => !p.keep && !cleanedIds.has(p.id));
    if (toClean.length === 0) return;
    const addedMb = toClean.reduce((sum, p) => sum + p.ramMb, 0);
    setCleanedIds((prev) => {
      const next = new Set(prev);
      toClean.forEach((p) => next.add(p.id));
      return next;
    });
    setFreedTotalMb((prev) => prev + addedMb);
  };

  const handleRestore = () => {
    setCleanedIds(new Set());
    setFreedTotalMb(0);
  };

  const startStreamDemo = () => {
    setStreamChars(0);
    setIsStreaming(true);
  };

  const pendingReclaimMb = calculateFreedRamMb(
    processes.filter((p) => !cleanedIds.has(p.id))
  );

  const tabs: Array<{ id: TuiTab; label: string }> = [
    { id: 'splash', label: copy.tabs.splash },
    { id: 'focus', label: copy.tabs.focus },
    { id: 'hud', label: copy.tabs.hud },
  ];

  const responseText = SAMPLE_RESPONSE[locale].slice(0, streamChars);
  const progressRatio = streamChars / Math.max(1, SAMPLE_RESPONSE[locale].length);
  const liveTokSec = isStreaming ? (24.5 + progressRatio * 4.2).toFixed(1) : '28.4';

  return (
    <motion.div
      id="simulator"
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.38, delay: 0.08, ease: [0.16, 1, 0.3, 1] }}
      className="flex flex-col rounded-xl border border-white/10 bg-[#0B0F16] overflow-hidden shadow-2xl shadow-black/50"
    >
      {/* Window Header */}
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-white/10 bg-[#0F1520] px-4 py-3">
        <span className="font-mono text-xs font-medium text-slate-300 whitespace-nowrap shrink-0">
          llmLab TUI
        </span>

        <div
          role="tablist"
          aria-label="TUI preview screens"
          className="inline-flex max-w-full flex-wrap items-center gap-1 rounded-lg bg-[#07090D] p-1 border border-white/5"
        >
          {tabs.map((tab) => {
            const active = activeTab === tab.id;
            return (
              <motion.button
                key={tab.id}
                type="button"
                role="tab"
                aria-selected={active}
                onClick={() => setActiveTab(tab.id)}
                whileHover={{ y: -1 }}
                whileTap={{ scale: 0.97 }}
                transition={{ duration: 0.14 }}
                className={`relative px-2.5 py-1 text-xs font-medium rounded-md whitespace-nowrap shrink-0 transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#00F5D4] ${
                  active ? 'text-white' : 'text-slate-400 hover:text-slate-200'
                }`}
              >
                {active && (
                  <motion.span
                    layoutId="tui-tab-pill"
                    transition={{ type: 'spring', stiffness: 460, damping: 34 }}
                    className="absolute inset-0 rounded-md bg-[#1B2638] border border-white/10"
                  />
                )}
                <span className="relative z-10">{tab.label}</span>
              </motion.button>
            );
          })}
        </div>
      </div>

      {/* Console Viewport */}
      <div className="p-4 sm:p-5 min-h-[340px] flex flex-col justify-between bg-[#07090D]">
        <AnimatePresence mode="wait">
          {activeTab === 'splash' && (
            <motion.div
              key="splash"
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ duration: 0.18 }}
              className="flex flex-col gap-4"
            >
              <div className="grid grid-cols-1 md:grid-cols-12 gap-4 items-center">
                <div className="md:col-span-6 flex flex-col items-center justify-center rounded-lg border border-white/5 bg-[#0A0E14] py-4 px-2 overflow-hidden">
                  <pre
                    aria-label="llmLab console logo"
                    className="font-mono text-[8px] sm:text-[8.5px] leading-[1.12] select-none"
                  >
                    {LOGO_LINES.map((line, idx) => {
                      const visible = idx < scanRow;
                      const isHead = scanRow < LOGO_LINES.length && idx + 1 === scanRow;
                      const ratio = idx / (LOGO_LINES.length - 1);
                      const r = Math.round(0 + 58 * ratio);
                      const g = Math.round(245 + (134 - 245) * ratio);
                      const b = Math.round(212 + (255 - 212) * ratio);
                      const inVisor = idx >= 5 && idx <= 12;

                      return (
                        <span
                          key={idx}
                          className="block transition-opacity duration-75"
                          style={{
                            opacity: visible ? 1 : 0.06,
                            color: isHead
                              ? '#FFFFFF'
                              : pulseActive && inVisor
                                ? '#FFFFFF'
                                : `rgb(${r}, ${g}, ${b})`,
                          }}
                        >
                          {line}
                        </span>
                      );
                    })}
                  </pre>
                  <p className="mt-2.5 font-mono text-[8px] sm:text-[9px] tracking-[0.04em] text-[#A5DCFF] whitespace-nowrap text-center">
                    l l m L a b · L O C A L A I R U N T I M E
                  </p>
                </div>

                <div className="md:col-span-6 rounded-lg border border-white/10 bg-[#0D1219] p-3.5 font-mono text-[11px] space-y-2">
                  <p className="text-white font-semibold pb-1.5 border-b border-white/10">
                    Hardware Probe
                  </p>
                  <div className="flex justify-between gap-2 whitespace-nowrap">
                    <span className="text-slate-400">CPU</span>
                    <span className="text-[#00F5D4]">i5-12400H · AVX2</span>
                  </div>
                  <div className="flex justify-between gap-2 whitespace-nowrap">
                    <span className="text-slate-400">Cores</span>
                    <span className="text-slate-200">4P / 8E (16T)</span>
                  </div>
                  <div className="flex justify-between gap-2 whitespace-nowrap">
                    <span className="text-slate-400">GPU</span>
                    <span className="text-[#3A86FF]">Iris Xe · Vulkan</span>
                  </div>
                  <div className="flex justify-between gap-2 whitespace-nowrap">
                    <span className="text-slate-400">RAM</span>
                    <span className="text-emerald-300">5.4 / 16.0 GB</span>
                  </div>
                  <div className="flex justify-between gap-2 pt-1.5 border-t border-white/10 whitespace-nowrap">
                    <span className="text-slate-400">Budget</span>
                    <span className="text-white">~9.3 GB GGUF</span>
                  </div>
                </div>
              </div>

              <div className="flex items-center justify-between gap-3 pt-1">
                <span className="font-mono text-xs text-slate-400">
                  1.4s · Enter / Space / --fast
                </span>
                <motion.button
                  type="button"
                  onClick={replayScan}
                  whileHover={{ scale: 1.03, y: -1 }}
                  whileTap={{ scale: 0.96 }}
                  className="inline-flex items-center gap-1.5 rounded-lg border border-white/15 bg-[#121925] px-3.5 py-1.5 text-xs font-medium text-white hover:border-[#00F5D4]/60 whitespace-nowrap shrink-0 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#00F5D4]"
                >
                  <Play className="h-3.5 w-3.5 text-[#00F5D4]" aria-hidden="true" />
                  <span>{copy.replayBtn}</span>
                </motion.button>
              </div>
            </motion.div>
          )}

          {activeTab === 'focus' && (
            <motion.div
              key="focus"
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ duration: 0.18 }}
              className="flex flex-col gap-3"
            >
              <div className="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-white/10 bg-[#0D1219] px-3.5 py-2.5 font-mono text-xs">
                <span className="text-slate-300">
                  {copy.freedLabel}:{' '}
                  <strong className="text-[#00F5D4] tabular-nums">
                    +{formatRamGb(pendingReclaimMb)}
                  </strong>
                </span>
                <span className="text-emerald-300 tabular-nums">
                  Freed: +{formatRamGb(freedTotalMb)}
                </span>
              </div>

              <div className="divide-y divide-white/5 rounded-lg border border-white/10 bg-[#0A0E14] font-mono text-xs">
                {processes.map((proc) => {
                  const isClosed = cleanedIds.has(proc.id);
                  return (
                    <motion.button
                      key={proc.id}
                      type="button"
                      disabled={isClosed}
                      onClick={() => toggleKeep(proc.id)}
                      whileHover={isClosed ? undefined : { backgroundColor: 'rgba(255,255,255,0.03)' }}
                      whileTap={isClosed ? undefined : { scale: 0.995 }}
                      className={`w-full flex items-center justify-between gap-3 px-3.5 py-2 text-left transition-opacity ${
                        isClosed ? 'opacity-45 cursor-default' : 'cursor-pointer'
                      }`}
                    >
                      <div className="flex items-center gap-3 min-w-0">
                        <span
                          className={`shrink-0 ${
                            proc.keep ? 'text-[#00F5D4]' : 'text-amber-300'
                          }`}
                        >
                          {proc.keep ? '[x]' : '[ ]'}
                        </span>
                        <span className="truncate text-slate-200">{proc.title}</span>
                      </div>
                      <div className="flex items-center gap-3 shrink-0 tabular-nums">
                        <span className="text-slate-400">{proc.ramMb} MB</span>
                        <span
                          className={
                            isClosed
                              ? 'text-emerald-400'
                              : proc.keep
                                ? 'text-[#00F5D4]'
                                : 'text-amber-300'
                          }
                        >
                          {isClosed ? 'Freed' : proc.keep ? 'Keep' : 'Close'}
                        </span>
                      </div>
                    </motion.button>
                  );
                })}
              </div>

              <div className="flex flex-wrap items-center justify-between gap-2 pt-1">
                <span className="inline-flex items-center gap-1.5 text-xs text-slate-400">
                  <ShieldCheck className="h-3.5 w-3.5 text-[#00F5D4] shrink-0" aria-hidden="true" />
                  <span>{copy.protectedNote}</span>
                </span>

                <div className="flex items-center gap-2 shrink-0">
                  <motion.button
                    type="button"
                    onClick={handleRestore}
                    whileHover={{ scale: 1.03 }}
                    whileTap={{ scale: 0.96 }}
                    className="inline-flex items-center gap-1.5 rounded-lg border border-white/15 bg-[#121925] px-3 py-1.5 text-xs font-medium text-slate-200 hover:text-white whitespace-nowrap shrink-0"
                  >
                    <RotateCcw className="h-3.5 w-3.5" aria-hidden="true" />
                    <span>{copy.restoreBtn}</span>
                  </motion.button>
                  <motion.button
                    type="button"
                    onClick={handleApplyCleanup}
                    whileHover={{ scale: 1.03, y: -1 }}
                    whileTap={{ scale: 0.96 }}
                    className="inline-flex items-center gap-1.5 rounded-lg bg-[#00F5D4] px-3.5 py-1.5 text-xs font-semibold text-[#07090D] hover:bg-[#19FFE0] whitespace-nowrap shrink-0"
                  >
                    <span>{copy.cleanBtn}</span>
                  </motion.button>
                </div>
              </div>
            </motion.div>
          )}

          {activeTab === 'hud' && (
            <motion.div
              key="hud"
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ duration: 0.18 }}
              className="flex flex-col justify-between gap-4 h-full"
            >
              <div className="rounded-lg border border-white/10 bg-[#0D1219] px-3.5 py-2.5 font-mono text-xs flex flex-wrap items-center justify-between gap-2">
                <span className="text-[#00F5D4] font-semibold tabular-nums">
                  {liveTokSec} tok/s
                </span>
                <span className="text-slate-300 tabular-nums">TTFT: 142 ms</span>
                <span className="text-slate-300 tabular-nums">RAM: 4.1 / 16.0 GB</span>
                <span className="text-[#3A86FF] tabular-nums">CPU: 74%</span>
              </div>

              <div className="rounded-lg border border-white/10 bg-[#0A0E14] p-4 font-mono text-xs leading-relaxed space-y-3 min-h-[175px]">
                <p className="text-[#3A86FF]">
                  You &gt;{' '}
                  <span className="text-slate-200">
                    {locale === 'ru'
                      ? 'Как оптимизирован запуск Qwen 2.5 3B?'
                      : 'How is Qwen 2.5 3B tuned on this laptop?'}
                  </span>
                </p>
                <p className="text-[#00F5D4]">
                  AI &gt;{' '}
                  <span className="text-slate-200">{responseText}</span>
                  {isStreaming && (
                    <span className="inline-block w-1.5 h-3.5 ml-1 align-middle bg-[#00F5D4] animate-pulse" />
                  )}
                </p>
              </div>

              <div className="flex items-center justify-between gap-3">
                <span className="font-mono text-xs text-slate-400">
                  llama-server · qwen2.5-3b-q4_k_m.gguf
                </span>
                <motion.button
                  type="button"
                  onClick={startStreamDemo}
                  whileHover={{ scale: 1.03, y: -1 }}
                  whileTap={{ scale: 0.96 }}
                  className="inline-flex items-center gap-1.5 rounded-lg bg-[#3A86FF] px-3.5 py-1.5 text-xs font-semibold text-white hover:bg-[#2E75EB] whitespace-nowrap shrink-0"
                >
                  <Sparkles className="h-3.5 w-3.5" aria-hidden="true" />
                  <span>{copy.streamPromptBtn}</span>
                </motion.button>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </motion.div>
  );
}
