import { useEffect, useState } from 'react';
import { AnimatePresence, motion } from 'motion/react';
import { Download, Play, RotateCcw, Search, ShieldCheck, Wrench } from 'lucide-react';
import {
  calculateFreedRamMb,
  formatRamGb,
  HF_MODEL_SAMPLES,
  INITIAL_FOCUS_PROCESSES,
  Locale,
  LOGO_LINES,
  ProcessSample,
  SITE_COPY,
} from '../data/llmlabContent';

type TuiTab = 'splash' | 'hf' | 'agent' | 'focus' | 'hud';

interface TuiSimulatorProps {
  locale: Locale;
}

const SAMPLE_RESPONSE: Record<Locale, string> = {
  ru: 'План питания переключён на High Performance. Потоки привязаны к 6 P-ядрам, KV-кэш квантован в q8_0. Скорость генерации: 28.4 tok/s.',
  en: 'Power plan switched to High Performance. Threads pinned to 6 P-cores with q8_0 KV cache. Generation rate: 28.4 tok/s.',
};

const AGENT_TRACE: Record<Locale, string[]> = {
  ru: [
    '[Skill: web_search] "python tray monitor ram psutil" -> 4 результата',
    '[Skill: write_file] ./workspace/ram_monitor.py (создан, 1.4 КБ)',
    '[Skill: run_command] python -m py_compile ram_monitor.py -> exit=0',
  ],
  en: [
    '[Skill: web_search] "python tray monitor ram psutil" -> 4 results',
    '[Skill: write_file] ./workspace/ram_monitor.py (saved, 1.4 KB)',
    '[Skill: run_command] python -m py_compile ram_monitor.py -> exit=0',
  ],
};

export function TuiSimulator({ locale }: TuiSimulatorProps) {
  const [activeTab, setActiveTab] = useState<TuiTab>('splash');
  const [scanRow, setScanRow] = useState<number>(LOGO_LINES.length);
  const [pulseActive, setPulseActive] = useState(false);

  // HF Hub state
  const [hfFilter, setHfFilter] = useState('');
  const [selectedHfId, setSelectedHfId] = useState(HF_MODEL_SAMPLES[0].id);
  const [downloadPct, setDownloadPct] = useState(0);
  const [isDownloading, setIsDownloading] = useState(false);

  // Agent Skills state
  const [skillWeb, setSkillWeb] = useState(true);
  const [skillBuild, setSkillBuild] = useState(true);
  const [skillShell, setSkillShell] = useState(false);
  const [agentStep, setAgentStep] = useState(AGENT_TRACE[locale].length);
  const [isRunningAgent, setIsRunningAgent] = useState(false);

  // Focus Mode state
  const [processes, setProcesses] = useState<ProcessSample[]>(INITIAL_FOCUS_PROCESSES);
  const [cleanedIds, setCleanedIds] = useState<Set<string>>(new Set());
  const [freedTotalMb, setFreedTotalMb] = useState(0);

  // Chat + HUD state
  const [streamChars, setStreamChars] = useState(SAMPLE_RESPONSE[locale].length);
  const [isStreaming, setIsStreaming] = useState(false);

  const copy = SITE_COPY[locale].simulator;

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

  useEffect(() => {
    if (!isDownloading) return;
    if (downloadPct >= 100) {
      setIsDownloading(false);
      return;
    }
    const timer = window.setTimeout(() => {
      setDownloadPct((prev) => Math.min(100, prev + 14));
    }, 90);
    return () => window.clearTimeout(timer);
  }, [isDownloading, downloadPct]);

  useEffect(() => {
    if (!isRunningAgent) return;
    const total = AGENT_TRACE[locale].length;
    if (agentStep >= total) {
      setIsRunningAgent(false);
      return;
    }
    const timer = window.setTimeout(() => {
      setAgentStep((prev) => prev + 1);
    }, 320);
    return () => window.clearTimeout(timer);
  }, [isRunningAgent, agentStep, locale]);

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

  const startDownloadDemo = () => {
    setDownloadPct(8);
    setIsDownloading(true);
  };

  const startAgentDemo = () => {
    setAgentStep(0);
    setIsRunningAgent(true);
  };

  const pendingReclaimMb = calculateFreedRamMb(
    processes.filter((p) => !cleanedIds.has(p.id))
  );

  const visibleHfModels = HF_MODEL_SAMPLES.filter((m) =>
    m.repo.toLowerCase().includes(hfFilter.trim().toLowerCase())
  );

  const tabs: Array<{ id: TuiTab; label: string }> = [
    { id: 'splash', label: copy.tabs.splash },
    { id: 'hf', label: copy.tabs.hf },
    { id: 'agent', label: copy.tabs.agent },
    { id: 'focus', label: copy.tabs.focus },
    { id: 'hud', label: copy.tabs.hud },
  ];

  const responseText = SAMPLE_RESPONSE[locale].slice(0, streamChars);
  const progressRatio = streamChars / Math.max(1, SAMPLE_RESPONSE[locale].length);
  const liveTokSec = isStreaming ? (24.5 + progressRatio * 4.2).toFixed(1) : '28.4';

  return (
    <motion.div
      id="simulator"
      initial={{ opacity: 0, y: 14 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.34, delay: 0.06, ease: [0.16, 1, 0.3, 1] }}
      className="flex flex-col rounded-lg border border-white/15 bg-[#0A0A0A] overflow-hidden"
    >
      {/* Window Header */}
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-white/15 bg-black px-4 py-3">
        <span className="font-mono text-xs font-medium text-neutral-300 whitespace-nowrap shrink-0">
          llmLab.exe · ←/→
        </span>

        <div
          role="tablist"
          aria-label="TUI preview screens"
          className="inline-flex max-w-full flex-wrap items-center gap-1 rounded-md bg-[#0A0A0A] p-1 border border-white/10"
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
                whileTap={{ scale: 0.96 }}
                transition={{ duration: 0.14 }}
                className={`relative px-2 py-1 text-[11px] font-medium rounded-sm whitespace-nowrap shrink-0 transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white ${
                  active ? 'text-black' : 'text-neutral-400 hover:text-white'
                }`}
              >
                {active && (
                  <motion.span
                    layoutId="tui-tab-pill"
                    transition={{ type: 'spring', stiffness: 460, damping: 34 }}
                    className="absolute inset-0 rounded-sm bg-white"
                  />
                )}
                <span className="relative z-10">{tab.label}</span>
              </motion.button>
            );
          })}
        </div>
      </div>

      {/* Console Viewport */}
      <div className="p-4 sm:p-5 min-h-[340px] flex flex-col justify-between bg-black">
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
                <div className="md:col-span-6 flex flex-col items-center justify-center rounded-md border border-white/10 bg-[#050505] py-4 px-2 overflow-hidden">
                  <pre
                    aria-label="llmLab console logo"
                    className="font-mono text-[8px] sm:text-[8.5px] leading-[1.12] select-none"
                  >
                    {LOGO_LINES.map((line, idx) => {
                      const visible = idx < scanRow;
                      const isHead = scanRow < LOGO_LINES.length && idx + 1 === scanRow;
                      const ratio = idx / (LOGO_LINES.length - 1);
                      const shade = Math.round(255 - ratio * 95);
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
                                : `rgb(${shade}, ${shade}, ${shade})`,
                          }}
                        >
                          {line}
                        </span>
                      );
                    })}
                  </pre>
                  <p className="mt-2.5 font-mono text-[8px] tracking-[0.12em] text-neutral-300 whitespace-nowrap text-center">
                    llmLab · LOCAL AI RUNTIME
                  </p>
                </div>

                <div className="md:col-span-6 rounded-md border border-white/15 bg-[#0A0A0A] p-3.5 font-mono text-[11px] space-y-2">
                  <p className="text-white font-semibold pb-1.5 border-b border-white/10">
                    Hardware Probe
                  </p>
                  <div className="flex justify-between gap-2 whitespace-nowrap">
                    <span className="text-neutral-500">CPU</span>
                    <span className="text-white">i5-12400H · AVX2</span>
                  </div>
                  <div className="flex justify-between gap-2 whitespace-nowrap">
                    <span className="text-neutral-500">Cores</span>
                    <span className="text-neutral-200">4P / 8E (16T)</span>
                  </div>
                  <div className="flex justify-between gap-2 whitespace-nowrap">
                    <span className="text-neutral-500">GPU</span>
                    <span className="text-neutral-200">Iris Xe · Vulkan</span>
                  </div>
                  <div className="flex justify-between gap-2 whitespace-nowrap">
                    <span className="text-neutral-500">RAM</span>
                    <span className="text-neutral-200">5.4 / 16.0 GB</span>
                  </div>
                  <div className="flex justify-between gap-2 pt-1.5 border-t border-white/10 whitespace-nowrap">
                    <span className="text-neutral-500">Budget</span>
                    <span className="text-white font-semibold">~9.3 GB GGUF</span>
                  </div>
                </div>
              </div>

              <div className="flex flex-wrap items-center justify-between gap-3 pt-1">
                <span className="font-mono text-xs text-neutral-500">
                  1.4s · Enter · <span className="whitespace-nowrap">--fast</span>
                </span>
                <motion.button
                  type="button"
                  onClick={replayScan}
                  whileHover={{ scale: 1.03, y: -1 }}
                  whileTap={{ scale: 0.95 }}
                  transition={{ type: 'spring', stiffness: 440, damping: 24 }}
                  className="inline-flex items-center gap-1.5 rounded-md border border-white/20 bg-black px-3.5 py-1.5 text-xs font-medium text-white hover:bg-white hover:text-black transition-colors whitespace-nowrap shrink-0"
                >
                  <Play className="h-3.5 w-3.5" aria-hidden="true" />
                  <span>{copy.replayBtn}</span>
                </motion.button>
              </div>
            </motion.div>
          )}

          {activeTab === 'hf' && (
            <motion.div
              key="hf"
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ duration: 0.18 }}
              className="flex flex-col gap-3 font-mono text-xs"
            >
              <div className="flex items-center gap-2 rounded-md border border-white/15 bg-[#0A0A0A] px-3 py-2">
                <Search className="h-3.5 w-3.5 text-neutral-400 shrink-0" aria-hidden="true" />
                <input
                  type="text"
                  value={hfFilter}
                  onChange={(e) => setHfFilter(e.target.value)}
                  placeholder="qwen coder / llama..."
                  aria-label="Search Hugging Face models"
                  className="w-full bg-transparent text-xs text-white placeholder:text-neutral-500 focus:outline-none"
                />
                <motion.button
                  type="button"
                  onClick={() => setHfFilter('')}
                  whileTap={{ scale: 0.95 }}
                  className="rounded border border-white/15 px-2 py-0.5 text-[10px] text-neutral-300 hover:bg-white hover:text-black whitespace-nowrap shrink-0"
                >
                  {copy.hfAutoPickBtn}
                </motion.button>
              </div>

              <div className="divide-y divide-white/10 rounded-md border border-white/15 bg-[#050505]">
                {visibleHfModels.map((model) => {
                  const selected = selectedHfId === model.id;
                  return (
                    <button
                      key={model.id}
                      type="button"
                      onClick={() => {
                        setSelectedHfId(model.id);
                        setDownloadPct(0);
                        setIsDownloading(false);
                      }}
                      className={`w-full flex items-center justify-between gap-2 px-3 py-2 text-left transition-colors ${
                        selected ? 'bg-white/10' : 'hover:bg-white/5'
                      }`}
                    >
                      <div className="min-w-0 flex items-center gap-2">
                        <span className={model.fitsPc ? 'text-white' : 'text-neutral-500'}>
                          {model.fit[locale]}
                        </span>
                        <span className="truncate text-neutral-200">{model.repo}</span>
                      </div>
                      <div className="flex items-center gap-3 shrink-0 tabular-nums text-[11px]">
                        <span className="text-neutral-400">{model.sizeGb.toFixed(1)} GB</span>
                        <span className="text-neutral-500 hidden sm:inline">{model.speed}</span>
                      </div>
                    </button>
                  );
                })}
              </div>

              <div className="space-y-2 pt-1">
                <div className="h-2 w-full rounded-sm bg-neutral-900 border border-white/10 overflow-hidden">
                  <div
                    className="h-full bg-white transition-all duration-100"
                    style={{ width: `${downloadPct}%` }}
                  />
                </div>
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <span className="text-[11px] text-neutral-400 tabular-nums">
                    {downloadPct === 100
                      ? 'Saved -> ./models/model-q4_k_m.gguf'
                      : isDownloading
                        ? `Downloading... ${downloadPct}% (42.8 MB/s)`
                        : 'Budget: ~9.3 GB GGUF · HF Hub Direct'}
                  </span>
                  <motion.button
                    type="button"
                    onClick={startDownloadDemo}
                    whileHover={{ scale: 1.03, y: -1 }}
                    whileTap={{ scale: 0.95 }}
                    className="inline-flex items-center gap-1.5 rounded-md bg-white px-3 py-1.5 text-xs font-semibold text-black hover:bg-neutral-200 whitespace-nowrap shrink-0"
                  >
                    <Download className="h-3.5 w-3.5" aria-hidden="true" />
                    <span>{copy.hfDownloadBtn}</span>
                  </motion.button>
                </div>
              </div>
            </motion.div>
          )}

          {activeTab === 'agent' && (
            <motion.div
              key="agent"
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ duration: 0.18 }}
              className="flex flex-col gap-3 font-mono text-xs"
            >
              <div className="flex flex-wrap gap-2">
                {[
                  { label: 'Web Search', active: skillWeb, toggle: () => setSkillWeb((v) => !v) },
                  { label: 'App Builder ./workspace', active: skillBuild, toggle: () => setSkillBuild((v) => !v) },
                  { label: 'Shell Runner', active: skillShell, toggle: () => setSkillShell((v) => !v) },
                ].map((sk) => (
                  <motion.button
                    key={sk.label}
                    type="button"
                    onClick={sk.toggle}
                    whileTap={{ scale: 0.95 }}
                    className={`rounded border px-2.5 py-1 text-[11px] transition-colors ${
                      sk.active
                        ? 'border-white bg-white text-black font-semibold'
                        : 'border-white/15 bg-black text-neutral-400 hover:text-white'
                    }`}
                  >
                    {sk.active ? '[x] ' : '[ ] '}
                    {sk.label}
                  </motion.button>
                ))}
              </div>

              <div className="rounded-md border border-white/15 bg-[#050505] p-3.5 space-y-2 min-h-[170px]">
                <p className="text-neutral-400">
                  Builder Agent &gt;{' '}
                  <span className="text-white">
                    {locale === 'ru'
                      ? 'Найди пример и собери утилиту мониторинга RAM в ./workspace'
                      : 'Search docs and build a RAM monitor script inside ./workspace'}
                  </span>
                </p>
                {AGENT_TRACE[locale].slice(0, agentStep).map((line, idx) => (
                  <p key={idx} className="text-neutral-300 text-[11px]">
                    {line}
                  </p>
                ))}
              </div>

              <div className="flex items-center justify-between gap-2">
                <span className="text-[11px] text-neutral-500">
                  ReAct Tools · DuckDuckGo + Sandbox FS
                </span>
                <motion.button
                  type="button"
                  onClick={startAgentDemo}
                  whileHover={{ scale: 1.03, y: -1 }}
                  whileTap={{ scale: 0.95 }}
                  className="inline-flex items-center gap-1.5 rounded-md bg-white px-3.5 py-1.5 text-xs font-semibold text-black hover:bg-neutral-200 whitespace-nowrap shrink-0"
                >
                  <Wrench className="h-3.5 w-3.5" aria-hidden="true" />
                  <span>{copy.agentRunBtn}</span>
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
              <div className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-white/15 bg-[#0A0A0A] px-3.5 py-2.5 font-mono text-xs">
                <span className="text-neutral-400">
                  {copy.freedLabel}:{' '}
                  <strong className="text-white tabular-nums">
                    +{formatRamGb(pendingReclaimMb)}
                  </strong>
                </span>
                <span className="text-white tabular-nums">
                  {copy.freedTotalLabel}: +{formatRamGb(freedTotalMb)}
                </span>
              </div>

              <div className="divide-y divide-white/10 rounded-md border border-white/15 bg-[#050505] font-mono text-xs">
                {processes.map((proc) => {
                  const isClosed = cleanedIds.has(proc.id);
                  return (
                    <motion.button
                      key={proc.id}
                      type="button"
                      disabled={isClosed}
                      onClick={() => toggleKeep(proc.id)}
                      whileHover={isClosed ? undefined : { backgroundColor: 'rgba(255,255,255,0.05)' }}
                      whileTap={isClosed ? undefined : { scale: 0.99 }}
                      className={`w-full flex items-center justify-between gap-3 px-3.5 py-2 text-left transition-opacity ${
                        isClosed ? 'opacity-40 cursor-default' : 'cursor-pointer'
                      }`}
                    >
                      <div className="flex items-center gap-3 min-w-0">
                        <span className="shrink-0 text-white">
                          {proc.keep ? '[x]' : '[ ]'}
                        </span>
                        <span
                          className={`truncate ${
                            isClosed ? 'line-through text-neutral-500' : 'text-neutral-200'
                          }`}
                        >
                          {proc.title}
                        </span>
                      </div>
                      <div className="flex items-center gap-3 shrink-0 tabular-nums">
                        <span className="text-neutral-500">{proc.ramMb} MB</span>
                        <span className={proc.keep ? 'text-white' : 'text-neutral-400'}>
                          {isClosed
                            ? copy.statusClosed
                            : proc.keep
                              ? copy.statusKeep
                              : copy.statusClose}
                        </span>
                      </div>
                    </motion.button>
                  );
                })}
              </div>

              <div className="flex flex-wrap items-center justify-between gap-2 pt-1">
                <span className="inline-flex items-center gap-1.5 text-xs text-neutral-400">
                  <ShieldCheck className="h-3.5 w-3.5 text-white shrink-0" aria-hidden="true" />
                  <span>{copy.protectedNote}</span>
                </span>

                <div className="flex items-center gap-2 shrink-0">
                  <motion.button
                    type="button"
                    onClick={handleRestore}
                    whileHover={{ scale: 1.03 }}
                    whileTap={{ scale: 0.95 }}
                    className="inline-flex items-center gap-1.5 rounded-md border border-white/20 bg-black px-3 py-1.5 text-xs font-medium text-white hover:bg-neutral-900 whitespace-nowrap shrink-0"
                  >
                    <RotateCcw className="h-3.5 w-3.5" aria-hidden="true" />
                    <span>{copy.restoreBtn}</span>
                  </motion.button>
                  <motion.button
                    type="button"
                    onClick={handleApplyCleanup}
                    whileHover={{ scale: 1.03, y: -1 }}
                    whileTap={{ scale: 0.95 }}
                    className="inline-flex items-center gap-1.5 rounded-md bg-white px-3.5 py-1.5 text-xs font-semibold text-black hover:bg-neutral-200 whitespace-nowrap shrink-0"
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
              <div className="rounded-md border border-white/15 bg-[#0A0A0A] px-3.5 py-2.5 font-mono text-xs flex flex-wrap items-center justify-between gap-2">
                <span className="text-white font-semibold tabular-nums">
                  {liveTokSec} tok/s
                </span>
                <span className="text-neutral-300 tabular-nums">TTFT: 142 ms</span>
                <span className="text-neutral-300 tabular-nums">RAM: 4.1 / 16.0 GB</span>
                <span className="text-neutral-300 tabular-nums">CPU: 74%</span>
              </div>

              <div className="rounded-md border border-white/15 bg-[#050505] p-4 font-mono text-xs leading-relaxed space-y-3 min-h-[175px]">
                <p className="text-neutral-400">
                  You &gt;{' '}
                  <span className="text-white">
                    {locale === 'ru'
                      ? 'Как настроен запуск Qwen 2.5 3B?'
                      : 'How is Qwen 2.5 3B tuned here?'}
                  </span>
                </p>
                <p className="text-white">
                  AI &gt;{' '}
                  <span className="text-neutral-200">{responseText}</span>
                  {isStreaming && (
                    <span className="inline-block w-1.5 h-3.5 ml-1 align-middle bg-white animate-pulse" />
                  )}
                </p>
              </div>

              <div className="flex items-center justify-between gap-3">
                <span className="font-mono text-xs text-neutral-500">
                  llama-server · qwen2.5-3b-q4_k_m.gguf
                </span>
                <motion.button
                  type="button"
                  onClick={startStreamDemo}
                  whileHover={{ scale: 1.03, y: -1 }}
                  whileTap={{ scale: 0.95 }}
                  className="inline-flex items-center gap-1.5 rounded-md bg-white px-3.5 py-1.5 text-xs font-semibold text-black hover:bg-neutral-200 whitespace-nowrap shrink-0"
                >
                  <Play className="h-3.5 w-3.5" aria-hidden="true" />
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
