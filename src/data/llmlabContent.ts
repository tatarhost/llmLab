export type Locale = 'ru' | 'en';
export type ShellMode = 'powershell' | 'cmd' | 'source';
export type EngineId = 'avx2' | 'vulkan' | 'openvino' | 'cloud';

export interface ProcessSample {
  id: string;
  name: string;
  title: string;
  ramMb: number;
  keep: boolean;
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
    case 'cmd':
      return `powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/${GITHUB_REPO}/main/install.ps1 | iex"`;
    case 'source':
      return `git clone ${GITHUB_URL}.git && cd llmLab && python scripts/make_icon.py && cargo build --release --target x86_64-pc-windows-msvc`;
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
      meta: 'Windows 10 / 11 x64 · Нативный Rust TUI · < 25 МБ RAM',
      title: 'Локальные ИИ-модели в консоли на максимальной скорости',
      subtitle:
        'Сверхлёгкая среда запуска GGUF-моделей для Windows без дискретной видеокарты. Управляет окнами для освобождения RAM, настраивает P-ядра и считает tok/s.',
      copyBtn: 'Копировать',
      copiedBtn: 'Скопировано',
      releaseBtn: 'Релизы GitHub',
      shellLabels: {
        powershell: 'PowerShell',
        cmd: 'CMD / Win+R',
        source: 'Из исходников',
      },
      commandHint: {
        powershell:
          'Скачивает llmLab-win64.zip, проверяет SHA256, прописывает PATH, создаёт ярлык на Рабочем столе и запускает llmLab.',
        cmd: 'Запуск установщика из обычной командной строки или диалога Win + R.',
        source: 'Сборка с генерацией multi-res icon.ico через Pillow и вшиванием в .exe.',
      },
    },
    simulator: {
      tabs: {
        splash: 'Запуск',
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
    },
    capabilities: {
      heading: 'Как устроен llmLab',
      lead: 'Один нативный бинарник llmLab.exe без Electron, Python-рантайма и фоновых процессов.',
      items: [
        {
          title: 'Режим «Только ИИ» с белым списком окон',
          detail:
            'Собирает открытые окна через EnumWindows и фоновые процессы тяжелее 80 МБ. Отмеченные [x] окна (Discord, Telegram, VS Code) остаются, остальное мягко закрывается по WM_CLOSE со снапшотом для возврата по Ctrl+R.',
          spec: 'keep_apps.json · session_snapshot.json',
        },
        {
          title: 'Auto-Tuner под гибридные CPU и питание',
          detail:
            'Читает топологию P-ядер и E-ядер через GetLogicalProcessorInformationEx, привязывает потоки к P-ядрам, включает High Performance (powercfg) и ставит HIGH_PRIORITY_CLASS.',
          spec: 'P-Core Threads · High Priority',
        },
        {
          title: 'Консольная анимация без мерцания',
          detail:
            'Двухбуферный рендер 60 FPS с построчным появлением логотипа, вспышкой центральной зоны </>, выводом параметров CPU/GPU/RAM и ASCII-фоллбэком для старого cmd.exe.',
          spec: '1.4s Startup · Enter / --fast',
        },
        {
          title: 'Потребление < 25 МБ RAM',
          detail:
            'Сборка с LTO и strip оставляет максимум оперативной памяти под саму GGUF-модель и KV-кэш q8_0.',
          spec: 'x86_64-pc-windows-msvc · LTO',
        },
      ],
    },
    engines: {
      heading: 'Движки и профили запуска',
      lead: 'Выберите бэкенд под вашу конфигурацию — от старого CPU с AVX2 до облачных API.',
      flagsLabel: 'Параметры запуска',
    },
    tutorial: {
      heading: 'Быстрый старт',
      steps: [
        {
          step: '01',
          title: 'Установите одной командой',
          body: 'Выполните команду PowerShell выше. Скрипт настроит PATH, создаст config.toml и папку models.',
        },
        {
          step: '02',
          title: 'Освободите RAM в Focus Mode',
          body: 'Нажмите [2] в меню (или llmLab --focus), отметьте нужные окна клавишей Space и нажмите Enter.',
        },
        {
          step: '03',
          title: 'Запустите модель или Cloud API',
          body: 'Поместите .gguf модель в models/ (рядом с llama-server.exe) или задайте LLMLAB_API_KEY для 300+ tok/s.',
        },
      ],
    },
    footer: {
      tagline: 'llmLab — нативная консольная среда для локального ИИ (Windows x64)',
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
      meta: 'Windows 10 / 11 x64 · Native Rust TUI · < 25 MB RAM',
      title: 'Local AI models in your terminal at full hardware speed',
      subtitle:
        'An ultra-lightweight GGUF runtime for Windows without a discrete GPU. Reclaims RAM while keeping chosen windows, pins threads to P-cores, and tracks tok/s live.',
      copyBtn: 'Copy',
      copiedBtn: 'Copied',
      releaseBtn: 'GitHub Releases',
      shellLabels: {
        powershell: 'PowerShell',
        cmd: 'CMD / Win+R',
        source: 'From Source',
      },
      commandHint: {
        powershell:
          'Downloads llmLab-win64.zip, verifies SHA256, updates PATH, creates a Desktop shortcut, and launches llmLab.',
        cmd: 'Runs the installer directly from Command Prompt or Win + R.',
        source: 'Builds locally and embeds a multi-res icon.ico via Pillow and winres.',
      },
    },
    simulator: {
      tabs: {
        splash: 'Startup',
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
    },
    capabilities: {
      heading: 'How llmLab works',
      lead: 'A single native llmLab.exe binary with zero Electron, Python runtime, or background daemons.',
      items: [
        {
          title: 'AI Focus Mode with Window Whitelist',
          detail:
            'Enumerates visible windows via EnumWindows and processes over 80 MB. Kept apps (Discord, Telegram, VS Code) stay open; everything else closes cleanly via WM_CLOSE with a Ctrl+R restore snapshot.',
          spec: 'keep_apps.json · session_snapshot.json',
        },
        {
          title: 'Hybrid CPU & Power Auto-Tuner',
          detail:
            'Reads exact P-cores and E-cores via GetLogicalProcessorInformationEx, pins threads to P-cores, enables High Performance via powercfg, and sets HIGH_PRIORITY_CLASS.',
          spec: 'P-Core Threads · High Priority',
        },
        {
          title: 'Flicker-Free Startup Probe',
          detail:
            'Double-buffered 60 FPS scanline reveal, </> visor flash, live CPU/GPU/RAM readout, and automatic ASCII fallback on legacy cmd.exe consoles.',
          spec: '1.4s Startup · Enter / --fast',
        },
        {
          title: 'Under 25 MB RAM Footprint',
          detail:
            'Compiled with LTO and symbol stripping so maximum memory remains available for your GGUF weights and q8_0 KV cache.',
          spec: 'x86_64-pc-windows-msvc · LTO',
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
          title: 'Install in one command',
          body: 'Run the PowerShell snippet above to configure PATH, create config.toml, and prepare models/.',
        },
        {
          step: '02',
          title: 'Free RAM in Focus Mode',
          body: 'Press [2] on the dashboard (or run llmLab --focus), mark apps to keep with Space, and press Enter.',
        },
        {
          step: '03',
          title: 'Run a GGUF model or Cloud API',
          body: 'Drop a .gguf file into models/ alongside llama-server.exe, or set LLMLAB_API_KEY for 300+ tok/s.',
        },
      ],
    },
    footer: {
      tagline: 'llmLab — native local AI terminal runtime for Windows x64',
      github: 'GitHub',
      releases: 'Releases',
    },
  },
};
