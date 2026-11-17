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
      ru: 'Оптимальный режим для ноутбуков без дискретной GPU. На гибридных Intel 12–14 Gen использует только быстрые P-ядра.',
      en: 'Best mode for laptops without a discrete GPU. Restricts threads to fast P-cores on hybrid Intel 12th–14th Gen CPUs.',
    },
    speedNote: {
      ru: 'Приоритет процесса HIGH_PRIORITY_CLASS + авто-переключение Windows в High Performance.',
      en: 'Runs with HIGH_PRIORITY_CLASS and temporarily switches Windows to High Performance power plan.',
    },
  },
  {
    id: 'vulkan',
    label: 'llama.cpp Vulkan',
    target: 'iGPU / AMD / NVIDIA',
    flags: '-ngl 99 -t <P-Cores> -c 4096 --flash-attn --cache-type-k q8_0',
    summary: {
      ru: 'Автоматически проверяет наличие vulkan-1.dll и переносит слои модели на встроенную графику Iris Xe, Radeon или дискретную карту.',
      en: 'Detects vulkan-1.dll automatically and offloads layers to Intel Iris Xe, AMD Radeon iGPU, or discrete GPUs.',
    },
    speedNote: {
      ru: 'Даёт заметный прирост скорости генерации токенов и снижает нагрузку на процессор.',
      en: 'Accelerates token generation throughput while freeing CPU headroom.',
    },
  },
  {
    id: 'openvino',
    label: 'OpenVINO GenAI',
    target: 'Intel Core / Arc / NPU',
    flags: '--device AUTO --max-context 4096',
    summary: {
      ru: 'Использует нативный стек Intel OpenVINO при наличии библиотек в системе для ускорения на процессорах и графике Intel.',
      en: 'Uses Intel OpenVINO runtime when installed to accelerate inference across Intel CPU and integrated graphics.',
    },
    speedNote: {
      ru: 'Автоматический выбор лучшего Intel-устройства через --device AUTO.',
      en: 'Automatic Intel execution device selection via --device AUTO.',
    },
  },
  {
    id: 'cloud',
    label: 'Cloud Boost',
    target: 'Groq / Cerebras / OpenRouter',
    flags: 'LLMLAB_API_KEY=<env>  ->  300+ tok/s SSE stream',
    summary: {
      ru: 'Потоковый OpenAI-совместимый клиент без нагрузки на оперативную память и батарею ноутбука.',
      en: 'Zero-footprint streaming OpenAI-compatible mode with zero local RAM or battery load.',
    },
    speedNote: {
      ru: 'Мгновенный TTFT и живой Speed HUD прямо внутри консоли.',
      en: 'Instant TTFT and live tok/s telemetry directly inside the terminal HUD.',
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
    overview: string;
    install: string;
    simulator: string;
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
    title: string;
    tabs: {
      splash: string;
      focus: string;
      hud: string;
    };
    replayBtn: string;
    cleanBtn: string;
    restoreBtn: string;
    freedLabel: string;
    protectedNote: string;
    streamPromptBtn: string;
  };
  capabilities: {
    heading: string;
    lead: string;
    items: Array<{
      num: string;
      title: string;
      detail: string;
      metric: string;
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
      overview: 'Обзор',
      simulator: 'Демо TUI',
      engines: 'Движки',
      install: 'Установка',
      cta: 'Получить команду',
    },
    hero: {
      meta: 'Windows 10 / 11 x64 · Нативный Rust TUI · < 25 МБ RAM',
      title: 'Локальный ИИ на максимум скорости — даже без видеокарты',
      subtitle:
        'Сверхлёгкая консольная среда для запуска GGUF-моделей. Освобождает оперативную память с сохранением нужных окон, настраивает P-ядра и замеряет tok/s в реальном времени.',
      copyBtn: 'Копировать команду',
      copiedBtn: 'Скопировано',
      releaseBtn: 'Релизы GitHub',
      shellLabels: {
        powershell: 'PowerShell',
        cmd: 'CMD / Win+R',
        source: 'Сборка из кода',
      },
      commandHint: {
        powershell:
          'Скачивает llmLab-win64.zip, проверяет SHA256, добавляет в PATH, создаёт ярлык на Рабочем столе и сразу запускает.',
        cmd: 'Запускает установщик прямо из обычной командной строки или диалога Win + R.',
        source: 'Локальная сборка с генерацией multi-res icon.ico через Pillow и вшиванием в .exe.',
      },
    },
    simulator: {
      title: 'Интерактивный предпросмотр консоли',
      tabs: {
        splash: 'Анимация запуска',
        focus: 'Режим «Только ИИ»',
        hud: 'Чат + Speed HUD',
      },
      replayBtn: 'Повторить 60 FPS скан',
      cleanBtn: 'Очистить RAM',
      restoreBtn: 'Восстановить (Ctrl+R)',
      freedLabel: 'Будет освобождено',
      protectedNote: 'Системные службы Windows, VPN, аудио и терминал жёстко защищены.',
      streamPromptBtn: 'Симулировать ответ',
    },
    capabilities: {
      heading: 'Спроектировано для слабого железа',
      lead: 'Никакого Electron, Python-рантайма или фоновых сервисов. Единый нативный llmLab.exe запускается мгновенно.',
      items: [
        {
          num: '01.',
          title: 'Режим «Только ИИ» с белым списком окон',
          detail:
            'Сканирует открытые окна через Win32 EnumWindows и фоновые процессы тяжелее 80 МБ. Вы отмечаете галочкой [x], что оставить (Discord, Telegram, VS Code), а остальное мягко закрывается через WM_CLOSE со снапшотом для возврата по Ctrl+R.',
          metric: 'keep_apps.json + session_snapshot.json',
        },
        {
          num: '02.',
          title: 'Auto-Tuner под гибридные CPU и питание',
          detail:
            'Определяет реальные P-ядра и E-ядра через GetLogicalProcessorInformationEx, ограничивает потоки быстрыми ядрами, временно переводит Windows в High Performance (powercfg) и запускает движок с HIGH_PRIORITY_CLASS.',
          metric: 'P-Core Threads + High Priority',
        },
        {
          num: '03.',
          title: 'TrueColor скан-анимация без мерцания',
          detail:
            'Двухбуферный рендер на 60 FPS: градиентное появление логотипа, белый импульс центрального выреза </>, проверка CPU/GPU/RAM и автоматический ASCII-фоллбэк в старом cmd.exe.',
          metric: '1.4s Startup · Enter / --fast skip',
        },
        {
          num: '04.',
          title: 'Крошечный вес и нулевой оверхед',
          detail:
            'Собрано с LTO и strip. Весь терминальный интерфейс занимает меньше 25 МБ RAM, оставляя максимум памяти под GGUF-модель и KV-кэш q8_0.',
          metric: '< 25 MB Runtime Footprint',
        },
      ],
    },
    engines: {
      heading: 'Профили движков и ускорение',
      lead: 'Выберите подходящий бэкенд под вашу конфигурацию — от старого CPU с AVX2 до облачных LPU на 300+ tok/s.',
      flagsLabel: 'Передаваемые параметры запуска',
    },
    tutorial: {
      heading: 'Быстрый старт за 3 шага',
      steps: [
        {
          step: 'Шаг 1',
          title: 'Установите одной командой',
          body: 'Выполните команду PowerShell из блока выше. Установщик сам настроит PATH, создаст config.toml и папку models.',
        },
        {
          step: 'Шаг 2',
          title: 'Очистите память перед запуском',
          body: 'Нажмите [2] в главном меню (или запустите llmLab --focus), оставьте нужные окна клавишей Space и примените очистку Enter.',
        },
        {
          step: 'Шаг 3',
          title: 'Подключите GGUF или Cloud API',
          body: 'Положите .gguf модель в models/ (рядом с llama-server.exe) или задайте LLMLAB_API_KEY для облачного режима 300+ tok/s.',
        },
      ],
    },
    footer: {
      tagline: 'llmLab — нативная консольная среда для локального ИИ под Windows x64',
      github: 'Репозиторий GitHub',
      releases: 'Скачать релиз',
    },
  },
  en: {
    nav: {
      overview: 'Overview',
      simulator: 'TUI Demo',
      engines: 'Engines',
      install: 'Install',
      cta: 'Get Command',
    },
    hero: {
      meta: 'Windows 10 / 11 x64 · Native Rust TUI · < 25 MB RAM',
      title: 'Maximum local AI speed — even without a dedicated GPU',
      subtitle:
        'An ultra-lightweight terminal runtime for GGUF models. Frees RAM while keeping your chosen windows, pins threads to fast P-cores, and streams live tok/s telemetry.',
      copyBtn: 'Copy Command',
      copiedBtn: 'Copied',
      releaseBtn: 'GitHub Releases',
      shellLabels: {
        powershell: 'PowerShell',
        cmd: 'CMD / Win+R',
        source: 'Build from Source',
      },
      commandHint: {
        powershell:
          'Downloads llmLab-win64.zip, verifies SHA256, adds to PATH, creates a Desktop shortcut, and launches immediately.',
        cmd: 'Runs the installer directly from legacy Command Prompt or the Win + R dialog.',
        source: 'Builds locally from source and embeds a multi-res icon.ico via Pillow and winres.',
      },
    },
    simulator: {
      title: 'Interactive Console Preview',
      tabs: {
        splash: 'Startup Animation',
        focus: 'AI Focus Mode',
        hud: 'Chat + Speed HUD',
      },
      replayBtn: 'Replay 60 FPS Scan',
      cleanBtn: 'Clean RAM',
      restoreBtn: 'Restore (Ctrl+R)',
      freedLabel: 'Reclaimable RAM',
      protectedNote: 'Core Windows services, VPNs, audio drivers, and your terminal stay protected.',
      streamPromptBtn: 'Simulate Stream',
    },
    capabilities: {
      heading: 'Engineered for constrained hardware',
      lead: 'Zero Electron, zero Python runtime overhead. A single native llmLab.exe starts instantaneously.',
      items: [
        {
          num: '01.',
          title: 'AI Focus Mode with Window Whitelist',
          detail:
            'Enumerates visible windows via Win32 EnumWindows and background processes over 80 MB. Mark apps to keep (Discord, Telegram, VS Code), gracefully close the rest via WM_CLOSE, and restore them in one click with Ctrl+R.',
          metric: 'keep_apps.json + session_snapshot.json',
        },
        {
          num: '02.',
          title: 'Hybrid CPU & Power Auto-Tuner',
          detail:
            'Reads exact P-cores vs E-cores via GetLogicalProcessorInformationEx, pins threads to P-cores, activates High Performance power plan via powercfg while active, and spawns with HIGH_PRIORITY_CLASS.',
          metric: 'P-Core Threads + High Priority',
        },
        {
          num: '03.',
          title: 'Flicker-Free 60 FPS Startup Scan',
          detail:
            'Double-buffered TrueColor scanline reveal with a white </> visor pulse, live CPU/GPU/RAM probe, and automatic ASCII fallback on legacy cmd.exe consoles.',
          metric: '1.4s Startup · Enter / --fast skip',
        },
        {
          num: '04.',
          title: 'Minimal Footprint, Maximum Model Budget',
          detail:
            'Compiled with LTO and symbol stripping. Consumes under 25 MB RAM so every available gigabyte goes to your quantized model and q8_0 KV cache.',
          metric: '< 25 MB Runtime Footprint',
        },
      ],
    },
    engines: {
      heading: 'Inference Engines & Acceleration',
      lead: 'Pick the backend suited for your machine — from AVX2 laptops to 300+ tok/s cloud endpoints.',
      flagsLabel: 'Auto-tuned launch flags',
    },
    tutorial: {
      heading: 'Up and running in 3 steps',
      steps: [
        {
          step: 'Step 1',
          title: 'Install with one command',
          body: 'Run the PowerShell snippet above. It sets up PATH, writes config.toml, and creates the models directory.',
        },
        {
          step: 'Step 2',
          title: 'Reclaim RAM before loading',
          body: 'Press [2] on the dashboard (or run llmLab --focus), toggle apps to keep with Space, and press Enter to clean RAM.',
        },
        {
          step: 'Step 3',
          title: 'Load a GGUF model or Cloud API',
          body: 'Drop any .gguf model into models/ alongside llama-server.exe, or set LLMLAB_API_KEY for 300+ tok/s cloud streaming.',
        },
      ],
    },
    footer: {
      tagline: 'llmLab — native local AI terminal runtime for Windows x64',
      github: 'GitHub Repository',
      releases: 'Download Release',
    },
  },
};
