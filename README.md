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
**llmLab** is an ultra-lightweight native terminal environment written in Rust (`< 25 MB` RAM footprint) designed for **Windows 10/11 PC**, **Linux (`x86_64` / `ARM64`)**, and **Android Termux**—capable of running local GGUF models, searching Hugging Face, and executing autonomous PC/mobile app-builder agent workflows.

### Key Features
- **Direct Hugging Face Hub & PC Auto-Match:** Searches GGUF models live via `huggingface.co/api`, scores compatibility (`[OK · Fast]` / `[Tight RAM]`) against your RAM/CPU/GPU budget, lists quantizations (`Q4_K_M`, `IQ4_XS`, `Q5_K_M`), and streams downloads directly into `models/`.
- **AI Agent, Skills & Web Search:** Autonomous ReAct tool loop (`AppBuilder`, `WebResearcher`, `FastAssistant`) with live DuckDuckGo web search, URL doc fetching, sandboxed `./workspace` app creation (`list_files`, `read_file`, `write_file`), and build shell commands.
- **Cross-Platform Arrow-Key TUI & Clean Upgrades:** Navigate tabs effortlessly with `←` / `→` (`Dashboard`, `HF Models`, `Agent / Chat`, `Skills`, `Focus Mode`, `Settings`), including stacked layout support for narrow Termux mobile screens. Running `install.ps1`, `install.sh`, or `llmLab --update` automatically stops any active instance and replaces the old binary cleanly.
- **AI Focus Mode (RAM & Window Cleaner):** Reclaims RAM cleanly on Windows (`WM_CLOSE`), Linux, and Termux (`SIGTERM` / `SIGKILL`) while strictly protecting OS services, shells, VPNs, and audio.
- **Hardware Auto-Tuner & Speed HUD:** Automatically sets `--threads` to fast P-cores, switches Windows power plan to High Performance while active, uses `HIGH_PRIORITY_CLASS`, and streams responses with real-time `tok/s` and `TTFT`.

---

### How to Run (Step-by-Step Tutorial)

#### Option 1: One-Command Install & Launch (Windows / Linux / Android Termux)
**Windows 10/11 (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/tatarhost/llmLab/main/install.ps1 | iex
```
**Linux (`x86_64` / `ARM64`) & Android Termux:**
```sh
curl -fsSL https://raw.githubusercontent.com/tatarhost/llmLab/main/install.sh | sh
```
Or from Windows `cmd.exe` / `Win + R`:
```cmd
powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/tatarhost/llmLab/main/install.ps1 | iex"
```
These installers automatically stop any running copy, remove the old version, download the matching release archive (`llmLab-win64.zip`, `llmLab-linux-x86_64.tar.gz`, `llmLab-linux-arm64.tar.gz`, or `llmLab-termux-arm64.tar.gz`), configure `PATH`, and launch `llmLab`.

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
  - `--models` — launch straight into the Hugging Face Model Hub.
  - `--focus` — launch straight into AI Focus Mode.
  - `--update` — delete the old binary and install the latest release.
  - `--ascii` — force ASCII logo fallback for legacy `cmd.exe` without UTF-8.
- **Hotkeys:**
  - `←` / `→` / `Tab` — switch between tabs (`Dashboard`, `HF Models`, `Agent / Chat`, `Skills`, `Focus Mode`, `Settings`)
  - `1` / `Enter` — launch model & open Agent Chat | `2` — HF Models | `3` — Skills | `4` — Focus Mode | `U` — Update | `Q` — Quit
  - In HF Models: `/` — search Hugging Face | `P` — hardware auto-match | `F` — pick quantization | `Enter` — download | `D` — delete local model
  - In Focus Mode: `Space` — toggle `[x] Keep` | `A` — select/deselect all | `Enter` — clean RAM | `F` — force-kill hung apps | `Ctrl+R` — restore closed apps

---

<a id="русский"></a>
## Русский

### Описание
**llmLab** — сверхлёгкая нативная консольная TUI-среда на Rust (`< 25 МБ` ОЗУ) для **ПК Windows 10/11**, **Linux (`x86_64` / `ARM64`)** и **Android Termux**: прямой поиск моделей на Hugging Face, автоподбор под железо, автономный ИИ-агент со скиллами и очистка ОЗУ.

### Основные возможности
- **Прямой Hugging Face Hub и автоподбор под устройство:** ищет GGUF-модели напрямую через `huggingface.co/api`, оценивает совместимость с RAM/CPU/GPU (`[OK · Быстро]` / `[Мало RAM]`), показывает квантования (`Q4_K_M`, `IQ4_XS`, `Q5_K_M`) и скачивает прямо в `models/`.
- **ИИ-агент со скиллами и поиском в интернете:** ReAct-цикл (`Builder Agent`, `Web Agent`, `Fast Chat`) с веб-поиском (DuckDuckGo), чтением страниц по URL, созданием проектов в песочнице `./workspace` и запуском команд сборки.
- **Адаптивный интерфейс на стрелках (включая узкий экран Termux):** переключайтесь между вкладками клавишами `←` / `→`, а на телефоне панели автоматически выстраиваются вертикально. При обновлении (`install.ps1`, `install.sh` или `llmLab --update`) старая версия удаляется автоматически.
- **Режим «Только ИИ» (Focus Mode):** освобождает ОЗУ на Windows (`WM_CLOSE`), Linux и Termux (`SIGTERM` / `SIGKILL`), защищая системные процессы, оболочку терминала и VPN.
- **Auto-Tuner + Speed HUD:** выставляет потоки по быстрым ядрам, включает `High Performance` на Windows и отображает `tok/s` и `TTFT`.

---

### Туториал: как запустить

#### Способ 1: Установка и обновление в одну команду (Windows / Linux / Termux)
**Windows 10/11 (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/tatarhost/llmLab/main/install.ps1 | iex
```
**Linux (`x86_64` / `ARM64`) и Android Termux:**
```sh
curl -fsSL https://raw.githubusercontent.com/tatarhost/llmLab/main/install.sh | sh
```
Или через Windows `cmd.exe` / `Win + R`:
```cmd
powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/tatarhost/llmLab/main/install.ps1 | iex"
```
Скрипт сам остановит запущенную копию, удалит старую версию бинарника, скачает нужный релиз (`llmLab-win64.zip`, `llmLab-linux-x86_64.tar.gz`, `llmLab-linux-arm64.tar.gz` или `llmLab-termux-arm64.tar.gz`), настроит `PATH` и запустит `llmLab`.

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
  - `--models` — сразу открыть каталог Hugging Face.
  - `--focus` — сразу открыть режим очистки ОЗУ «Только ИИ».
  - `--update` — удалить старую версию и установить свежий релиз с GitHub.
  - `--ascii` — принудительно включить ASCII-логотип для старого `cmd.exe` без UTF-8.
- **Горячие клавиши:**
  - `←` / `→` / `Tab` — переключение вкладок (`Обзор`, `Модели HF`, `Агент / Чат`, `Скиллы`, `Только ИИ`, `Настройки`)
  - `1` / `Enter` — запуск модели и Агент-чата | `2` — Модели HF | `3` — Скиллы | `4` — Только ИИ | `U` — Обновить | `Q` — Выход
  - В «Модели HF»: `/` — поиск по Hugging Face | `P` — автоподбор под ПК | `F` — выбор квантования | `Enter` — скачать | `D` — удалить локальную модель
  - В Focus Mode: `Space` — отметить `[x] Оставить` | `A` — выбрать/снять всё | `Enter` — очистить RAM | `F` — принудительно завершить зависшие процессы | `Ctrl+R` — восстановить закрытые окна
