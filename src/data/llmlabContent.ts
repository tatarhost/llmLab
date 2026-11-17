export type Locale = 'ru' | 'en';
export type ShellMode = 'powershell' | 'termux' | 'linux' | 'cmd' | 'source';
export type EngineId = 'avx2' | 'vulkan' | 'openvino' | 'cloud';

export interface ProcessSample {
  id: string;
  name: string;
  title: string;
  ramMb: number;
  keep: boolean;
}

export interface HfModelSample {
  id: string;
  repo: string;
  quant: string;
  sizeGb: number;
  fit: Record<Locale, string>;
  fitsPc: boolean;
  speed: string;
}

export interface EngineSpec {
  id: EngineId;
  label: string;
  target: string;
  flags: string;
  summary: Record<Locale, string>;
  speedNote: Record<Locale, string>;
}

export const GITHUB_REPO = 'tatarhost/llmLab';
export const GITHUB_URL = `https://github.com/${GITHUB_REPO}`;
export const RELEASES_URL = `${GITHUB_URL}/releases`;

export const LOGO_LINES: readonly string[] = [
  '█████████████▀        ▀█████████████',
  '████████████▀          ▀████████████',
  '████▀                           ████',
  '█████                         ▄█████',
  ' ▀████▄▄▄██████████████████▄▄█████▀ ',
  '   ▀███████████████   ▄█████████▀   ',
  '     ▀████▀ ▀█████    █▀ ▀████▀     ',
  '     ▄██▀    ▄███▀   █▄    ▀██▄     ',
  '   ▄███    ▄████▀   ████▄    ███▄   ',
  ' ▄█████▄    ▀███   ▄███▀    ▄█████▄ ',
  '█████████▄    █   ▄███    ▄█████████',
  '███████████▄▄█    █████▄▄███████████',
  '██████████████▄▄▄███████████████████',
  '██████████▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀██████████',
  '█████████                  █████████',
  '████████                    ████████',
  '███████                      ███████',
  '██████                        ██████',
];

export const INITIAL_FOCUS_PROCESSES: ProcessSample[] = [
  { id: 'chrome', name: 'chrome.exe', title: 'Google Chrome (18 tabs)', ramMb: 2840, keep: false },
  { id: 'steam', name: 'steamwebhelper.exe', title: 'Steam Client WebHelper', ramMb: 940, keep: false },
  { id: 'discord', name: 'Discord.exe', title: 'Discord', ramMb: 620, keep: true },
  { id: 'spotify', name: 'Spotify.exe', title: 'Spotify Desktop', ramMb: 410, keep: false },
  { id: 'code', name: 'Code.exe', title: 'Visual Studio Code', ramMb: 1180, keep: true },
  { id: 'telegram', name: 'Telegram.exe', title: 'Telegram Desktop', ramMb: 340, keep: true },
];

export const HF_MODEL_SAMPLES: HfModelSample[] = [
  {
    id: 'qwen-3b',
    repo: 'Qwen/Qwen2.5-3B-Instruct-GGUF',
    quant: 'Q4_K_M',
    sizeGb: 2.1,
    fit: { ru: '[OK · Быстро]', en: '[OK · Fast]' },
    fitsPc: true,
    speed: '18–28 tok/s',
  },
  {
    id: 'qwen-coder-3b',
    repo: 'Qwen/Qwen2.5-Coder-3B-Instruct-GGUF',
    quant: 'Q4_K_M',
    sizeGb: 2.1,
    fit: { ru: '[OK · Код]', en: '[OK · Code]' },
    fitsPc: true,
    speed: '18–28 tok/s',
  },
  {
    id: 'qwen-7b',
    repo: 'Qwen/Qwen2.5-7B-Instruct-GGUF',
    quant: 'Q4_K_M',
    sizeGb: 4.7,
    fit: { ru: '[OK · Норма]', en: '[OK · Fit]' },
    fitsPc: true,
    speed: '9–15 tok/s',
  },
  {
    id: 'qwen-14b',
    repo: 'bartowski/Qwen2.5-14B-Instruct-GGUF',
    quant: 'Q4_K_M',
    sizeGb: 9.0,
    fit: { ru: '[Мало RAM]', en: '[Tight RAM]' },
    fitsPc: false,
    speed: '< 4 tok/s',
  },
];

export const ENGINE_SPECS: EngineSpec[] = [
  {
    id: 'avx2',
    label: 'llama.cpp AVX2',
    target: 'CPU + RAM',
    flags: '-t <P-Cores> -c 4096 --flash-attn --cache-type-k q8_0 --cache-type-v q8_0',
    summary: {
      ru: 'Для ноутбуков и ПК без дискретной видеокарты. На гибридных процессорах Intel использует только быстрые P-ядра.',
      en: 'For laptops and desktops without a discrete GPU. Restricts threads to fast P-cores on hybrid Intel CPUs.',
    },
    speedNote: {
      ru: 'Приоритет процесса HIGH_PRIORITY_CLASS и временное переключение питания в High Performance.',
      en: 'Runs under HIGH_PRIORITY_CLASS and switches Windows to High Performance power plan while active.',
    },
  },
  {
    id: 'vulkan',
    label: 'llama.cpp Vulkan',
    target: 'iGPU / AMD / NVIDIA',
    flags: '-ngl 99 -t <P-Cores> -c 4096 --flash-attn --cache-type-k q8_0',
    summary: {
      ru: 'Проверяет vulkan-1.dll и переносит вычисления на встроенную графику Iris Xe, Radeon или дискретную видеокарту.',
      en: 'Detects vulkan-1.dll and offloads layers to Intel Iris Xe, AMD Radeon iGPU, or discrete GPUs.',
    },
    speedNote: {
      ru: 'Увеличивает скорость генерации и снижает загрузку CPU.',
      en: 'Increases token throughput while reducing CPU load.',
    },
  },
  {
    id: 'openvino',
    label: 'OpenVINO GenAI',
    target: 'Intel Core / Arc / NPU',
    flags: '--device AUTO --max-context 4096',
    summary: {
      ru: 'Использует установленный стек Intel OpenVINO для ускорения на процессорах и интегрированной графике Intel.',
      en: 'Uses installed Intel OpenVINO libraries for Intel CPU and integrated graphics execution.',
    },
    speedNote: {
      ru: 'Автоматический выбор устройства через --device AUTO.',
      en: 'Automatic Intel device routing via --device AUTO.',
    },
  },
  {
    id: 'cloud',
    label: 'Cloud Boost',
    target: 'Groq / Cerebras / OpenRouter',
    flags: 'LLMLAB_API_KEY=<env>  ->  300+ tok/s SSE stream',
    summary: {
      ru: 'Потоковый OpenAI-совместимый режим без нагрузки на оперативную память и батарею.',
      en: 'Streaming OpenAI-compatible mode with zero local RAM or battery load.',
    },
    speedNote: {
      ru: 'Мгновенный TTFT и живой счётчик tok/s в консоли.',
      en: 'Low TTFT and live tok/s telemetry directly in the console HUD.',
    },
  },
];

export function getInstallCommand(mode: ShellMode): string {
  switch (mode) {
    case 'powershell':
      return `irm https://raw.githubusercontent.com/${GITHUB_REPO}/main/install.ps1 | iex`;
    case 'termux':
      return `curl -fsSL https://raw.githubusercontent.com/${GITHUB_REPO}/main/install.sh | sh`;
    case 'linux':
      return `curl -fsSL https://raw.githubusercontent.com/${GITHUB_REPO}/main/install.sh | sh`;
    case 'cmd':
      return `powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/${GITHUB_REPO}/main/install.ps1 | iex"`;
    case 'source':
      return `git clone ${GITHUB_URL}.git && cd llmLab && cargo build --release`;
  }
}

export function calculateFreedRamMb(processes: readonly ProcessSample[]): number {
  return processes
    .filter((p) => !p.keep)
    .reduce((sum, p) => sum + p.ramMb, 0);
}

export function formatRamGb(mb: number): string {
  return `${(mb / 1024).toFixed(2)} GB`;
}

export interface SiteCopy {
  nav: {
    install: string;
    simulator: string;
    overview: string;
    engines: string;
    cta: string;
  };
  hero: {
    meta: string;
    title: string;
    subtitle: string;
    copyBtn: string;
    copiedBtn: string;
    releaseBtn: string;
    shellLabels: Record<ShellMode, string>;
    commandHint: Record<ShellMode, string>;
  };
  simulator: {
    tabs: {
      splash: string;
      hf: string;
      agent: string;
      focus: string;
      hud: string;
    };
    replayBtn: string;
    cleanBtn: string;
    restoreBtn: string;
    freedLabel: string;
    freedTotalLabel: string;
    statusClosed: string;
    statusKeep: string;
    statusClose: string;
    protectedNote: string;
    streamPromptBtn: string;
    hfAutoPickBtn: string;
    hfDownloadBtn: string;
    agentRunBtn: string;
  };
  capabilities: {
    heading: string;
    lead: string;
    items: Array<{
      title: string;
      detail: string;
      spec: string;
    }>;
  };
  engines: {
    heading: string;
    lead: string;
    flagsLabel: string;
  };
  tutorial: {
    heading: string;
    steps: Array<{
      step: string;
      title: string;
      body: string;
    }>;
  };
  footer: {
    tagline: string;
    github: string;
    releases: string;
  };
}

export const SITE_COPY: Record<Locale, SiteCopy> = {
  ru: {
    nav: {
      install: 'Установка',
      simulator: 'Демо TUI',
      overview: 'Возможности',
      engines: 'Движки',
      cta: 'Команда',
    },
    hero: {
      meta: 'Windows PC · Linux · Android Termux · Rust AI Agent',
      title: 'Локальный ИИ-агент и Hugging Face для ПК, Linux и Termux',
      subtitle:
        'Единый лёгкий Rust TUI для Windows, Linux (x86_64/ARM64) и Android Termux: прямой поиск GGUF с Hugging Face, автоподбор под железо, веб-поиск, агент-сборщик в workspace и очистка RAM.',
      copyBtn: 'Копировать',
      copiedBtn: 'Скопировано',
      releaseBtn: 'Релизы GitHub',
      shellLabels: {
        powershell: 'Windows (PS)',
        termux: 'Termux',
        linux: 'Linux',
        cmd: 'CMD',
        source: 'Исходники',
      },
      commandHint: {
        powershell:
          'Windows 10/11: удаляет старый llmLab.exe, скачивает свежий релиз, настраивает PATH и запускает.',
        termux:
          'Android Termux (ARM64): удаляет старый бинарник, скачивает llmLab-termux-arm64 (или собирает через pkg rust) и запускает.',
        linux:
          'Linux (x86_64 / ARM64): устанавливает или обновляет ~/.local/bin/llmLab с автоматической заменой старой версии.',
        cmd: 'Установка или обновление на Windows из cmd.exe или окна Win + R.',
        source: 'Локальная release-сборка из исходников (cargo build --release) для Windows / Linux / Termux.',
      },
    },
    simulator: {
      tabs: {
        splash: 'Запуск',
        hf: 'Модели HF',
        agent: 'Агент + Web',
        focus: 'Только ИИ',
        hud: 'Чат + HUD',
      },
      replayBtn: 'Повторить скан',
      cleanBtn: 'Очистить RAM',
      restoreBtn: 'Вернуть (Ctrl+R)',
      freedLabel: 'Освободится',
      freedTotalLabel: 'Освобождено',
      statusClosed: 'Закрыто',
      statusKeep: 'Оставить',
      statusClose: 'Закрыть',
      protectedNote: 'Системные службы Windows, VPN, аудио и терминал защищены.',
      streamPromptBtn: 'Запустить пример',
      hfAutoPickBtn: 'Автоподбор под ПК',
      hfDownloadBtn: 'Скачать GGUF',
      agentRunBtn: 'Запустить задачу агента',
    },
    capabilities: {
      heading: 'Как устроен llmLab',
      lead: 'Нативный Rust TUI с навигацией по стрелкам (←/→ вкладки, ↑/↓ списки) и автоматической заменой старой версии при обновлении.',
      items: [
        {
          title: 'Прямой Hugging Face Hub и автоподбор под ПК',
          detail:
            'Ищет GGUF-репозитории напрямую через Hugging Face API, анализирует свободный бюджет RAM/CPU/GPU, ранжирует совместимость ([OK · Быстро] / [Мало RAM]) и скачивает выбранное квантование с прогресс-баром.',
          spec: 'huggingface.co/api · Auto-Match · Streaming .gguf',
        },
        {
          title: 'ИИ-Агент: скиллы, субагенты и веб-поиск',
          detail:
            'Встроенный ReAct-цикл превращает локальную или облачную модель в агента: поиск в интернете (DuckDuckGo), чтение документации по URL, создание проектов в песочнице ./workspace и запуск сборки.',
          spec: 'App Builder · Web Search · Sandboxed ./workspace',
        },
        {
          title: 'Интерфейс на стрелках + Режим «Только ИИ»',
          detail:
            'Переключайтесь между вкладками клавишами ←/→, управляйте списками ↑/↓. Режим «Только ИИ» закрывает лишние процессы по WM_CLOSE со снапшотом возврата по Ctrl+R.',
          spec: '←/→ Tabs · ↑/↓ Lists · Ctrl+R Restore',
        },
        {
          title: 'Чистое обновление с удалением старой версии',
          detail:
            'При повторном запуске установщика или флага llmLab --update старый процесс останавливается, прежний llmLab.exe и временные файлы удаляются, а config.toml и скачанные модели сохраняются.',
          spec: 'llmLab --update · Clean Binary Replace',
        },
      ],
    },
    engines: {
      heading: 'Движки и профили запуска',
      lead: 'Автоматический подбор потоков P-ядер и параметров памяти под вашу конфигурацию.',
      flagsLabel: 'Параметры запуска',
    },
    tutorial: {
      heading: 'Быстрый старт',
      steps: [
        {
          step: '01',
          title: 'Установите или обновитесь одной командой',
          body: 'Запустите команду PowerShell выше — скрипт удалит старую версию, скачает свежий релиз и настроит PATH.',
        },
        {
          step: '02',
          title: 'Подберите и скачайте модель с Hugging Face',
          body: 'Перейдите стрелкой → во вкладку «Модели HF», нажмите P для автоподбора или / для поиска, затем Enter для загрузки.',
        },
        {
          step: '03',
          title: 'Включите скиллы агента и веб-поиск',
          body: 'Во вкладке «Скиллы» активируйте веб-поиск и сборку приложений в ./workspace, затем поставьте задачу во вкладке «Агент / Чат».',
        },
      ],
    },
    footer: {
      tagline: 'llmLab — нативный локальный ИИ-агент и TUI для Windows, Linux и Termux',
      github: 'GitHub',
      releases: 'Релизы',
    },
  },
  en: {
    nav: {
      install: 'Install',
      simulator: 'Preview',
      overview: 'Capabilities',
      engines: 'Engines',
      cta: 'Get Command',
    },
    hero: {
      meta: 'Windows PC · Linux · Android Termux · Rust AI Agent',
      title: 'Local AI Agent & Hugging Face Hub for Windows, Linux & Termux',
      subtitle:
        'One ultra-light Rust TUI for Windows PC, Linux (x86_64/ARM64), and Android Termux: direct Hugging Face GGUF search, hardware auto-selection, web search, workspace app builder, and RAM Focus Mode.',
      copyBtn: 'Copy',
      copiedBtn: 'Copied',
      releaseBtn: 'GitHub Releases',
      shellLabels: {
        powershell: 'Windows (PS)',
        termux: 'Termux',
        linux: 'Linux',
        cmd: 'CMD',
        source: 'Source',
      },
      commandHint: {
        powershell:
          'Windows 10/11: removes old llmLab.exe, downloads latest release, checks SHA256, configures PATH, and runs.',
        termux:
          'Android Termux (ARM64): removes old binary, downloads llmLab-termux-arm64 (or builds via pkg rust), and runs.',
        linux:
          'Linux (x86_64 / ARM64): installs or cleanly upgrades ~/.local/bin/llmLab with old-version cleanup.',
        cmd: 'Install or upgrade on Windows from Command Prompt or Win + R.',
        source: 'Build natively from source (cargo build --release) on Windows, Linux, or Termux.',
      },
    },
    simulator: {
      tabs: {
        splash: 'Startup',
        hf: 'HF Models',
        agent: 'Agent + Web',
        focus: 'Focus Mode',
        hud: 'Chat + HUD',
      },
      replayBtn: 'Replay Scan',
      cleanBtn: 'Clean RAM',
      restoreBtn: 'Restore (Ctrl+R)',
      freedLabel: 'Reclaimable',
      freedTotalLabel: 'Freed',
      statusClosed: 'Freed',
      statusKeep: 'Keep',
      statusClose: 'Close',
      protectedNote: 'Core Windows services, VPNs, audio, and terminal stay protected.',
      streamPromptBtn: 'Run Sample',
      hfAutoPickBtn: 'Auto-Match PC',
      hfDownloadBtn: 'Download GGUF',
      agentRunBtn: 'Run Agent Task',
    },
    capabilities: {
      heading: 'How llmLab works',
      lead: 'Native arrow-key TUI (←/→ tabs, ↑/↓ lists) with direct Hugging Face downloads and clean old-binary replacement on upgrade.',
      items: [
        {
          title: 'Direct Hugging Face Hub & PC Auto-Picker',
          detail:
            'Searches GGUF models live via Hugging Face API, scores hardware fit ([OK · Fast] / [Tight RAM]) against available RAM/CPU/GPU, and streams quantizations directly into ./models.',
          spec: 'huggingface.co/api · Auto-Match · Streaming .gguf',
        },
        {
          title: 'AI Agent with Skills, Sub-Agents & Web Search',
          detail:
            'Turns local or cloud LLMs into a tool-using agent: DuckDuckGo web search, URL doc fetching, sandboxed ./workspace file creation, and build command execution.',
          spec: 'App Builder · Web Search · Sandboxed ./workspace',
        },
        {
          title: 'Arrow-Key GUI & AI Focus Mode',
          detail:
            'Switch tabs cleanly with ←/→ and navigate lists with ↑/↓. Focus Mode reclaims RAM via WM_CLOSE while preserving selected apps and Ctrl+R restoration.',
          spec: '←/→ Tabs · ↑/↓ Lists · Ctrl+R Restore',
        },
        {
          title: 'Clean Old-Version Removal on Update',
          detail:
            'Running the installer again or llmLab --update stops the running process, deletes the old llmLab.exe binary, and keeps your config.toml and downloaded models intact.',
          spec: 'llmLab --update · Clean Binary Replace',
        },
      ],
    },
    engines: {
      heading: 'Engines & Profiles',
      lead: 'Select a backend suited to your hardware — from AVX2 CPUs to cloud endpoints.',
      flagsLabel: 'Launch flags',
    },
    tutorial: {
      heading: 'Quick Start',
      steps: [
        {
          step: '01',
          title: 'Install or upgrade in one command',
          body: 'Run the PowerShell snippet above to remove any old version, install the latest release, and configure PATH.',
        },
        {
          step: '02',
          title: 'Auto-pick & download from Hugging Face',
          body: 'Press → to open HF Models, press P for hardware auto-match or / to search, then Enter to download.',
        },
        {
          step: '03',
          title: 'Enable Agent Skills & Web Search',
          body: 'Toggle web search and ./workspace builder in Skills, then give your agent a task in Agent / Chat.',
        },
      ],
    },
    footer: {
      tagline: 'llmLab — native local AI agent & TUI for Windows, Linux & Termux',
      github: 'GitHub',
      releases: 'Releases',
    },
  },
};
