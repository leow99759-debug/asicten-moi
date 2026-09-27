# Как запустить разработку Jarvis через Claude Opus (Claude Code)

## Что в комплекте
| Файл | Для чего | Кто читает |
|---|---|---|
| `SPEC.md` | Полное ТЗ по видео (весь функционал, UI, голос, ИИ, приёмка) | Opus, только нужный раздел |
| `PLAN.md` | задачи по этапам M0–M13, чек-лист прогресса | Opus, каждую итерацию |
| `CLAUDE.md` | 6 строк контекста, грузится автоматически | Opus |
| `.claude/skills/jarvis-dev/SKILL.md` | Скилл разработки: цикл, тесты, git, экономия токенов | Opus |
| `LOOP_PROMPT.md` | Промпт одной итерации для цикла | Цикл |
| `STATE.md` | Короткая «память» (≤30 строк, перезаписывается) | Opus создаёт сам |

Служебные файлы (скилл, план, цикл) написаны по-английски: так они короче в токенах, а читаются на каждой итерации. ТЗ на русском, чтобы ты мог его читать и править.

## Шаг 1. Подготовка ПК (один раз)
Установи: Git, GitHub CLI (`gh auth login`), Rust (rustup, toolchain msvc), Node.js LTS, Visual Studio Build Tools (C++), Claude Code. WebView2 уже есть в Windows 10/11.

## Шаг 2. Репозиторий
Создай пустой репозиторий на GitHub (лучше **приватный**: голос Джарвиса защищён правами). Склонируй и положи туда все файлы из архива (вместе с папкой `.claude`).

## Шаг 3. Разрешения, чтобы цикл не останавливался на каждом вопросе
Создай `.claude/settings.json`:
```json
{
  "permissions": {
    "allow": ["Bash(cargo:*)", "Bash(npm:*)", "Bash(npx:*)", "Bash(git:*)", "Bash(gh:*)", "Bash(rg:*)", "Bash(pwsh:*)", "Bash(powershell:*)", "Edit", "Write", "Read", "WebFetch", "WebSearch"],
    "deny": ["Bash(git push --force:*)", "Bash(shutdown:*)", "Bash(rm -rf /:*)"]
  }
}
```

## Шаг 4. Первый запуск: вставь в Claude Code (модель Opus)
```
Ты ведущий разработчик проекта Jarvis: офлайн голосового ассистента для Windows 10/11, копии функционала из референсных видео.
Репозиторий: <ССЫЛКА_НА_РЕПО>. В корне лежат SPEC.md, PLAN.md, CLAUDE.md, LOOP_PROMPT.md и скилл .claude/skills/jarvis-dev.
Проект личный, приватный, некоммерческий: код и ресурсы из референсов можно брать (с комментарием-источником).
Главный референс архитектуры: https://github.com/Priler/jarvis
Референс хлопков/профилей окон: https://github.com/MateuszMlynekHub/launcher
Фразы Джарвиса (русский дубляж, 61 реплика): https://github.com/FitoDomik/Jarvis-Sound
Мой ПК: ASUS TUF Gaming F15, 8 ГБ RAM (в простое занято ~7 ГБ), RTX 3060 Laptop 6 ГБ, Windows 11. Память — главное ограничение. Ядро без LLM.

Сейчас сделай подготовку (T000):
1. Прочитай CLAUDE.md, скилл jarvis-dev, PLAN.md и SPEC.md §0, §1, §12 (остальное только по мере задач).
2. Проверь тулчейн (rustc, cargo, node, npm, gh, git, MSVC build tools). Если чего-то нет, скажи мне одной строкой и жди.
3. Создай .gitignore (target, node_modules, dist, assets/models, voices/**/*.wav, *.gguf, *.onnx, .env), README (≤20 строк), STATE.md по формату из скилла.
4. Если видишь в ТЗ противоречие или решение лучше (легче по RAM, надёжнее на Windows), запиши 1 строкой в STATE.md decisions и используй его.
5. Коммит "T000: spec and workflow", push.
Потом ничего не пиши, я запущу цикл.
```

## Шаг 5. Запуск цикла (Opus работает сам, задача за задачей)
В Claude Code один раз установи плагин цикла от Anthropic:
```
/plugin marketplace add anthropics/claude-code
/plugin install ralph-wiggum@claude-code-plugins
```
Запусти:
```
/ralph-loop "Read LOOP_PROMPT.md and do exactly what it says." --completion-promise "JARVIS_V1_DONE" --max-iterations 150
```
Остановить: `/cancel-ralph`. Продолжить потом: та же команда, прогресс хранится в PLAN.md/STATE.md и git.

**Запасной вариант** (если плагин не работает на Windows), PowerShell-цикл: каждая итерация в свежем контексте, это ещё экономнее по токенам:
```powershell
for ($i=1; $i -le 150; $i++) {
  $out = claude -p "Read LOOP_PROMPT.md and do exactly what it says." --model opus
  $out | Select-Object -Last 3
  if ($out -match "JARVIS_V1_DONE") { break }
}
```

## Шаг 6. Где нужен ты
- Задачи с пометкой `[!] needs-user` в PLAN.md (например, путь к твоему музыкальному приложению или запуск обучения голоса T120 на твоей RTX 3060 — перед ним закрой Chrome и игры).
- Голос Opus ищет и собирает сам (Jarvis-Sound, Priler, сайты с voice pack). Если захочешь точнее — нарежь ещё фраз из фильма и положи WAV в `tools/voice-train/data/raw/`.
- После каждого этапа (тег `m1`, `m2`…) скачай сборку из GitHub Actions и проверь руками по таблице SPEC §11.
- Готовая программа: **Releases → Jarvis-Full-Setup.exe** в твоём приватном репо. Скачал, установил, работает без интернета.
- ИИ-модуль (чат с локальной нейросетью в видеопамяти RTX 3060) — опционально, после v1.0:
  `/ralph-loop "Read LOOP_PROMPT.md. M13 is enabled now: work until all M13 tasks are [x], then output JARVIS_AI_DONE." --completion-promise "JARVIS_AI_DONE" --max-iterations 30`


## Важно
- Репозиторий держи приватным: проект личный, голос из фильма и чужой код внутри.
- Модели и звуки лежат не в самом git (чтобы он не весил гигабайты), а в Releases того же репо и внутри установщика.
