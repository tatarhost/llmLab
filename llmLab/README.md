# llmLab — Local AI Runtime (Windows x64 TUI)

Сверхлёгкая нативная консольная TUI-среда на Rust для запуска локальных LLM на слабых ПК и ноутбуках (CPU / RAM / iGPU) с минимальным потреблением ОЗУ (< 25 МБ).

## Запуск по одной команде (Windows PowerShell)
После публикации релиза на GitHub любой пользователь может установить и запустить `llmLab` одной командой:
```powershell
irm https://raw.githubusercontent.com/OWNER/llmLab/main/install.ps1 | iex
```
Скрипт автоматически скачает последний `llmLab-win64.zip` из GitHub Releases, проверит `SHA256`, распакует в `%LOCALAPPDATA%\llmLab`, добавит в `PATH`, создаст ярлык на рабочем столе с иконкой `llmLab` и сразу запустит приложение.

## Возможности
- **60 FPS анимация запуска**: построчное TrueColor-сканирование логотипа (`#00F5D4` → `#3A86FF`), белый импульс центрального выреза `</>` и живая диагностика CPU (P/E-ядра, AVX2), GPU (DXGI, Vulkan, OpenVINO) и доступной RAM. Пропуск по `Enter`/`Space` или `--fast`.
- **Режим «Только ИИ» (Focus Mode)**: показывает активные окна и фоновые процессы (`> 80 МБ`) с жёсткой защитой критических системных служб Windows, сохраняет белый список в `%USERPROFILE%\.llmlab\keep_apps.json`, отправляет корректный `WM_CLOSE`, считает освобождённую память в реальном времени и позволяет вернуть закрытые приложения в один клик (`Ctrl+R` или при выходе).
- **Auto-Tuner**: автоматически подбирает `--threads` по числу быстрых P-ядер, активирует план питания `High Performance` на время работы модели и запускает движок с `HIGH_PRIORITY_CLASS`.
- **Движки**: `llama.cpp` (AVX2 / Vulkan), `OpenVINO`, `bitnet.cpp`, а также облачный режим (Groq / Cerebras / OpenRouter) для 300+ tok/s.
- **Чат + Speed HUD**: отображает `tok/s`, `TTFT`, занятую RAM и загрузку CPU в реальном времени.

## Быстрый старт (Windows)
1. Поместите `assets/logo.jpg` в папку `assets/` (или используйте встроенный SVG-фоллбэк).
2. Сгенерируйте многоразмерную иконку `.ico`:
   ```powershell
   pip install pillow
   python scripts/make_icon.py
   ```
3. Соберите релизный бинарник:
   ```powershell
   cargo build --release --target x86_64-pc-windows-msvc
   ```
4. Положите `.gguf` модели в `./models` и `llama-server.exe` в PATH (или укажите путь в `config.toml`), затем запустите:
   ```powershell
   .\target\x86_64-pc-windows-msvc\release\llmLab.exe
   ```

## Флаги CLI
- `--fast` — пропустить анимацию запуска.
- `--focus` — сразу открыть режим очистки памяти «Только ИИ».
- `--ascii` — принудительно включить ASCII-логотип для старого `cmd.exe`.
