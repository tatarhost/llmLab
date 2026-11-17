import { useState } from 'react';
import { AnimatePresence, motion } from 'motion/react';
import { Check, Copy, ExternalLink, Terminal } from 'lucide-react';
import {
  getInstallCommand,
  Locale,
  RELEASES_URL,
  ShellMode,
  SITE_COPY,
} from '../data/llmlabContent';

interface CommandBoxProps {
  locale: Locale;
}

const SHELL_MODES: ShellMode[] = ['powershell', 'cmd', 'source'];

export function CommandBox({ locale }: CommandBoxProps) {
  const [shellMode, setShellMode] = useState<ShellMode>('powershell');
  const [copied, setCopied] = useState(false);

  const copy = SITE_COPY[locale].hero;
  const command = getInstallCommand(shellMode);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(command);
    } catch {
      const textarea = document.createElement('textarea');
      textarea.value = command;
      textarea.style.position = 'fixed';
      textarea.style.opacity = '0';
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand('copy');
      document.body.removeChild(textarea);
    }
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1800);
  };

  return (
    <motion.div
      id="install"
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.32, ease: [0.16, 1, 0.3, 1] }}
      className="rounded-xl border border-white/10 bg-[#0D1219] p-4 sm:p-5 shadow-2xl shadow-black/40"
    >
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-white/10 pb-3">
        <div
          role="tablist"
          aria-label="Shell installation method"
          className="inline-flex max-w-full flex-wrap items-center gap-1 rounded-lg bg-[#07090D] p-1 border border-white/5"
        >
          {SHELL_MODES.map((mode) => {
            const active = shellMode === mode;
            return (
              <motion.button
                key={mode}
                type="button"
                role="tab"
                aria-selected={active}
                onClick={() => {
                  setShellMode(mode);
                  setCopied(false);
                }}
                whileHover={{ y: -1 }}
                whileTap={{ scale: 0.97 }}
                transition={{ duration: 0.14 }}
                className={`relative px-2.5 sm:px-3.5 py-1.5 text-xs font-medium rounded-md whitespace-nowrap shrink-0 transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#00F5D4] ${
                  active ? 'text-[#07090D]' : 'text-slate-300 hover:text-white'
                }`}
              >
                {active && (
                  <motion.span
                    layoutId="shell-tab-indicator"
                    transition={{ type: 'spring', stiffness: 460, damping: 34 }}
                    className="absolute inset-0 rounded-md bg-[#00F5D4]"
                  />
                )}
                <span className="relative z-10">{copy.shellLabels[mode]}</span>
              </motion.button>
            );
          })}
        </div>

        <motion.a
          href={RELEASES_URL}
          target="_blank"
          rel="noreferrer"
          whileHover={{ y: -1, color: '#00F5D4' }}
          whileTap={{ scale: 0.97 }}
          transition={{ duration: 0.15 }}
          className="inline-flex items-center gap-1.5 text-xs font-medium text-slate-300 hover:text-[#00F5D4] whitespace-nowrap shrink-0 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#00F5D4]"
        >
          <span>{copy.releaseBtn}</span>
          <ExternalLink className="h-3.5 w-3.5" aria-hidden="true" />
        </motion.a>
      </div>

      {/* Command Surface: clean unbroken horizontal line on desktop, scrollable on narrow mobile */}
      <div className="mt-4 rounded-lg bg-[#07090D] border border-white/10 p-3.5 sm:p-4 space-y-3">
        <div className="flex items-center gap-2.5 overflow-x-auto pb-0.5">
          <Terminal className="h-4 w-4 shrink-0 text-[#00F5D4]" aria-hidden="true" />
          <AnimatePresence mode="wait">
            <motion.code
              key={shellMode}
              initial={{ opacity: 0, y: 4 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -4 }}
              transition={{ duration: 0.16, ease: [0.16, 1, 0.3, 1] }}
              data-testid="install-command-text"
              className="font-mono text-xs text-[#E6EDF5] whitespace-nowrap select-all"
            >
              {command}
            </motion.code>
          </AnimatePresence>
        </div>

        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 pt-2.5 border-t border-white/5">
          <p className="text-xs leading-relaxed text-slate-400">
            {copy.commandHint[shellMode]}
          </p>

          <motion.button
            type="button"
            onClick={handleCopy}
            whileHover={{ scale: 1.03, y: -1 }}
            whileTap={{ scale: 0.95 }}
            transition={{ type: 'spring', stiffness: 420, damping: 24 }}
            className={`inline-flex items-center justify-center gap-2 rounded-lg px-4 py-2 text-xs font-semibold whitespace-nowrap shrink-0 transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#00F5D4] ${
              copied
                ? 'bg-[#00F5D4] text-[#07090D]'
                : 'bg-[#3A86FF] text-white hover:bg-[#2E75EB]'
            }`}
          >
            <AnimatePresence mode="wait" initial={false}>
              {copied ? (
                <motion.span
                  key="copied"
                  initial={{ opacity: 0, scale: 0.85 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.85 }}
                  transition={{ duration: 0.14 }}
                  className="inline-flex items-center gap-1.5"
                >
                  <Check className="h-3.5 w-3.5" aria-hidden="true" />
                  <span>{copy.copiedBtn}</span>
                </motion.span>
              ) : (
                <motion.span
                  key="copy"
                  initial={{ opacity: 0, scale: 0.85 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.85 }}
                  transition={{ duration: 0.14 }}
                  className="inline-flex items-center gap-1.5"
                >
                  <Copy className="h-3.5 w-3.5" aria-hidden="true" />
                  <span>{copy.copyBtn}</span>
                </motion.span>
              )}
            </AnimatePresence>
          </motion.button>
        </div>
      </div>
    </motion.div>
  );
}
