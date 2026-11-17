# llmLab — Local AI Runtime (Windows x64 TUI)

> **Language / Язык:** [English](#english) · [Русский](#русский)

```text
█████████████▀        ▀█████████████
████████████▀          ▀████████████
████▀                           ████
█████                         ▄█████
 ▀████▄▄▄██████████████████▄▄█████▀
   ▀███████████████   ▄█████████▀
     ▀████▀ ▀█████    █▀ ▀████▀
     ▄██▀    ▄███▀   █▄    ▀██▄
   ▄███    ▄████▀   ████▄    ███▄
 ▄█████▄    ▀███   ▄███▀    ▄█████▄
█████████▄    █   ▄███    ▄█████████
███████████▄▄█    █████▄▄███████████
██████████████▄▄▄███████████████████
██████████▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀██████████
█████████                  █████████
████████                    ████████
███████                      ███████
██████                        ██████
  l l m L a b   //   L O C A L   A I   R U N T I M E
```

---

<a id="english"></a>
## English

### Overview
**llmLab** is an ultra-lightweight native Windows 10/11 x64 terminal environment (written in Rust, `< 25 MB` RAM footprint) designed to run local LLMs at maximum speed on low-end PCs and laptops—including systems without a discrete GPU (CPU / RAM / iGPU only).

### Key Features
- **60 FPS Startup Animation:** Double-buffered TrueColor scanline logo (`#00F5D4` → `#3A86FF`) with a white `</>` pulse and live hardware probe (CPU P/E-cores, AVX2/NEON, DXGI GPU, Vulkan/OpenVINO availability, and usable GGUF RAM budget). Skip anytime with `Enter`/`Space` or `--fast`.
- **AI Focus Mode (RAM & Window Cleaner):** Lists active application windows and background processes (`> 80 MB`) while strictly protecting critical Windows services, audio drivers, VPNs, and your terminal host. Toggle what to keep with `Space`, save your whitelist to `%USERPROFILE%\.llmlab\keep_apps.json`, close apps gracefully via `WM_CLOSE`, track freed RAM live, and restore everything in one click (`Ctrl+R` or on exit).
- **Hardware Auto-Tuner:** Automatically sets `--threads` to fast P-cores on hybrid Intel CPUs, temporarily switches Windows power plan to **High Performance** via `powercfg` (restoring your original plan on exit), and launches inference engines with `HIGH_PRIORITY_CLASS`.
- **Multi-Engine & Cloud Boost:** Supports `llama.cpp` (AVX2 & Vulkan), `OpenVINO`, `bitnet.cpp`, and OpenAI-compatible Cloud APIs (`Groq`, `Cerebras`, `OpenRouter`) for 300+ tok/s streaming.
- **Built-in Chat + Speed HUD:** Displays real-time `tok/s`, `TTFT` (Time To First Token), RAM usage, and CPU load while generating responses.

---

### How to Run (Step-by-Step Tutorial)

#### Option 1: One-Command Install & Launch (Recommended)
Open **PowerShell** on Windows 10/11 and run:
```powershell
irm https://raw.githubusercontent.com/OWNER/llmLab/main/install.ps1 | iex
```
Or from `cmd.exe` / `Win + R`:
```cmd
powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/OWNER/llmLab/main/install.ps1 | iex"
```
This automatically downloads the latest `llmLab-win64.zip` release, verifies `SHA256`, extracts it to `%LOCALAPPDATA%\llmLab`, adds it to your `PATH`, creates a Desktop shortcut, and launches `llmLab`.

#### Option 2: Manual Download
1. Download `llmLab-win64.zip` from the **Releases** page.
2. Extract it to any folder (e.g., `C:\Tools\llmLab\`).
3. Copy `config.example.toml` to `config.toml`.
4. Run `llmLab.exe`.

#### Option 3: Connecting a Local Model or Cloud Boost
- **Local GGUF model:**
  1. Download `llama-server.exe` (from `llama.cpp` Windows releases) and place it next to `llmLab.exe` or in your `PATH`.
  2. Put your `.gguf` model file (e.g., `qwen2.5-3b-instruct-q4_k_m.gguf`) into the `models/` folder.
  3. Launch `llmLab`, optionally press `2` to free RAM in **Focus Mode**, then select your model with `↑`/`↓` and press `Enter` or `1`.
- **Cloud Boost (300+ tok/s without local GPU/RAM load):**
  1. Set your API key in PowerShell:
     ```powershell
     $env:LLMLAB_API_KEY = "your_api_key_here"
     ```
  2. Open `llmLab`, press `3` (**Settings**), switch **Engine** to `Cloud Boost (300+ tok/s)`, pick a provider (`Groq` / `Cerebras` / `OpenRouter`), press `Esc`, and press `1` to start chatting.

#### Option 4: Build from Source
1. Install [Rust](https://rustup.rs/) (`x86_64-pc-windows-msvc`) and Python 3.10+.
2. Optional: place your custom `assets/logo.jpg` (otherwise the built-in `assets/logo.svg` fallback is used).
3. Generate the Windows icon and compile:
   ```powershell
   pip install pillow
   python scripts/make_icon.py
   cargo build --release --target x86_64-pc-windows-msvc
   ```
   The binary will be at `target\x86_64-pc-windows-msvc\release\llmLab.exe`.

### CLI Flags & Shortcuts
- **CLI Flags:**
  - `--fast` — skip the startup animation.
  - `--focus` — launch straight into AI Focus Mode.
  - `--ascii` — force ASCII logo fallback for legacy `cmd.exe` without UTF-8.
- **Hotkeys:**
  - `1` / `Enter` — launch model & open Chat | `2` — Focus Mode | `3` — Settings | `Q` — Quit
  - In Focus Mode: `Space` — toggle `[x] Keep` | `A` — select/deselect all | `Enter` — clean RAM | `F` — force-kill hung apps | `Ctrl+R` — restore closed apps

---

<a id="русский"></a>
## Русский

### Описание
**llmLab** — сверхлёгкая нативная консольная TUI-среда для Windows 10/11 x64 (написана на Rust, потребление ОЗУ `< 25 МБ`), созданная для запуска локальных ИИ-моделей с максимальной скоростью на слабых ПК и ноутбуках — в том числе без дискретной видеокарты (только CPU / RAM / iGPU).

### Основные возможности
- **Анимация запуска 60 FPS:** плавное TrueColor-сканирование логотипа (`#00F5D4` → `#3A86FF`) с белым импульсом центрального выреза `</>` и живой диагностикой железа (P/E-ядра CPU, AVX2/NEON, видеокарта через DXGI, доступность Vulkan/OpenVINO и оценка свободной памяти под GGUF). Пропускается по `Enter`/`Space` или флагом `--fast`.
- **Режим «Только ИИ» (Focus Mode):** показывает открытые окна и тяжёлые фоновые процессы (`> 80 МБ`) с жёсткой защитой критических служб Windows, аудиодрайверов, VPN и вашего терминала. Клавишей `Space` вы отмечаете, какие окна оставить (белый список сохраняется в `%USERPROFILE%\.llmlab\keep_apps.json`), мягко закрываете остальное через `WM_CLOSE`, видите счётчик освобождённой RAM и возвращаете закрытые программы в один клик (`Ctrl+R` или при выходе).
- **Auto-Tuner под процессор и питание:** на гибридных процессорах Intel выставляет `--threads` строго по числу быстрых P-ядер, автоматически переключает Windows в режим **Высокая производительность** (`powercfg`) на время инференса и запускает движок с приоритетом `HIGH_PRIORITY_CLASS`.
- **Поддержка локальных и облачных движков:** `llama.cpp` (AVX2 и Vulkan), `OpenVINO`, `bitnet.cpp`, а также облачное ускорение (`Groq`, `Cerebras`, `OpenRouter`) на 300+ tok/s.
- **Консольный чат + Speed HUD:** в реальном времени отображает `tok/s`, `TTFT` (время до первого токена), занятую RAM и загрузку CPU.

---

### Туториал: как запустить

#### Способ 1: Запуск по одной команде (рекомендуется)
Откройте **PowerShell** в Windows 10/11 и выполните:
```powershell
irm https://raw.githubusercontent.com/OWNER/llmLab/main/install.ps1 | iex
```
Или через `cmd.exe` / окно `Win + R`:
```cmd
powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/OWNER/llmLab/main/install.ps1 | iex"
```
Скрипт автоматически скачает последний релиз `llmLab-win64.zip`, проверит контрольную сумму `SHA256`, распакует файлы в `%LOCALAPPDATA%\llmLab`, пропишет программу в `PATH`, создаст ярлык на Рабочем столе и сразу запустит `llmLab`.

#### Способ 2: Ручная установка из Releases
1. Скачайте архив `llmLab-win64.zip` со страницы **Releases**.
2. Распакуйте его в удобную папку (например, `C:\Tools\llmLab\`).
3. Скопируйте `config.example.toml` в `config.toml`.
4. Запустите `llmLab.exe`.

#### Способ 3: Подключение локальной модели или облачного ускорения
- **Локальная GGUF-модель:**
  1. Скачайте `llama-server.exe` (из релизов `llama.cpp` для Windows) и положите рядом с `llmLab.exe` или добавьте в `PATH`.
  2. Положите файл модели `.gguf` (например, `qwen2.5-3b-instruct-q4_k_m.gguf`) в папку `models/`.
  3. Запустите `llmLab`. При необходимости нажмите `2`, чтобы освободить ОЗУ в **Focus Mode**, затем выберите модель стрелками `↑`/`↓` и нажмите `Enter` или `1`.
- **Облачное ускорение (300+ tok/s без нагрузки на ПК):**
  1. Задайте API-ключ в PowerShell:
     ```powershell
     $env:LLMLAB_API_KEY = "ваш_api_ключ"
     ```
  2. Запустите `llmLab`, нажмите `3` (**Настройки**), переключите **Движок инференса** на `Cloud Boost (300+ tok/s)`, выберите провайдера (`Groq` / `Cerebras` / `OpenRouter`), нажмите `Esc` и затем `1` для входа в чат.

#### Способ 4: Сборка из исходников
1. Установите [Rust](https://rustup.rs/) (`x86_64-pc-windows-msvc`) и Python 3.10+.
2. При желании положите свой логотип в `assets/logo.jpg` (иначе автоматически используется встроенный SVG-шаблон `assets/logo.svg`).
3. Сгенерируйте `.ico` и соберите релизный `.exe`:
   ```powershell
   pip install pillow
   python scripts/make_icon.py
   cargo build --release --target x86_64-pc-windows-msvc
   ```
   Готовый файл появится по пути `target\x86_64-pc-windows-msvc\release\llmLab.exe`.

### Флаги запуска и горячие клавиши
- **Флаги CLI:**
  - `--fast` — пропустить стартовую анимацию.
  - `--focus` — сразу открыть режим очистки ОЗУ «Только ИИ».
  - `--ascii` — принудительно включить ASCII-логотип для старого `cmd.exe` без UTF-8.
- **Горячие клавиши:**
  - `1` / `Enter` — запуск модели и чат | `2` — Focus Mode | `3` — Настройки | `Q` — Выход
  - В Focus Mode: `Space` — отметить `[x] Оставить` | `A` — выбрать/снять всё | `Enter` — очистить RAM | `F` — принудительно завершить зависшие процессы | `Ctrl+R` — восстановить закрытые окна
