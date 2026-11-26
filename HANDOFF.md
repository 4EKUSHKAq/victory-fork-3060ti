# HANDOFF — текущее состояние работ

Обновлено: 2026-11-26, 14:50 (Asia/Yekaterinburg)
Текущая ветка: `coolfox-rtx` (от `coolfox/main` = `codec-d/cool_fox` @ `f4b48ad`); наша прежняя ветка
`visual-and-nvidia-fixes` @ `4133084` **не тронута** и лежит в `myfork`.
Обе в приватном бэкапе `myfork` = https://github.com/4EKUSHKAq/victory-fork-3060ti
(вместе с `perf-and-upstream-fixes` и `master`).
§20: два тестовых прогона coolfox-rtx на 3060 Ti 8 ГБ (авторский `.cmd` = Series S и диагностический
Series X). Итог: **Series S даёт чёрный 3D-экран** (two-plane compose), Series X рисует картинку,
ambient работает (интерьер не чёрный), пересвет умеренный, но FPS **в 2–3 раза ниже нашей ветки**
(медиана 2–3 против 6, GPU-время 48–241 мс против 5 мс) и 3 compute-шейдера дропнуты (2 из них водные,
которые у нас переведены). Рекомендация: основная линия — `visual-and-nvidia-fixes`, coolfox-rtx —
экспериментальная, только с `--console series-x`.
§19: подключён публичный форк `codec-d/cool_fox` (667★, 627 коммитов, отдельная история), три
субагента разобрали лаунчер/pareto-пресет, перф-коммиты и свет; выбран **Путь А** — ветка `coolfox-rtx`
от их main (перенос их 900–1000-строчных модулей в нашу ветку нереален: 9/9 `git apply --check` = FAIL).
Добавлены `victory.toml`, `AGENTS.md`, `HANDOFF.md`; `cargo check --workspace --locked` exit 0, релиз
1 м 05 с, кэш отдельной папки `cache-pareto` прогрет за 294,3 с (25 663 переведено, 226 не переводятся:
195 GS + 26 wave32 + 5 shadow-mask). Команда запуска — §19.5.
§18 (ветка `visual-and-nvidia-fixes`): мониторинг всех 6 звёзд GitHub (за 48 ч — 2085 коммитов и 5849 PR), затем perf-батч **без сброса
кэша шейдеров**: #1237+#1245 (`VICTORY_LOOKUP_CACHE`), #1227 (`VICTORY_CARD_SYNC_BATCH`), host-часть
#1224 (`VICTORY_FILL_HOT`/`VICTORY_BURY_ASIDE`), VRAM-пара #1240+#1258 (`VICTORY_TEXTURE_MAX_DIM`,
`VICTORY_DYNAMIC_SLOTS`, `VICTORY_BIND_CACHE`, `VICTORY_RAM_TRIM_MB`) и свежий #1269 (94 % под текстуры
на картах ≤8 ГиБ). #1178 отклонён (6939 строк, 14 чужих коммитов, конфликт с нашим prefetch). Кэш
проверен: 25 637 шейдеров на месте, 0 переведено заново, перепрогрев 4,7 с.

## Что сделано на этом шаге

| # | Шаг | Статус |
|---|-----|--------|
| 0 | Изучены `AGENTS.md`, `README.md`, `docs/` (architecture, asset-policy, fork-ray-tracing) | готово |
| 0 | Проверено окружение (Git, Rust, cargo, rust-analyzer, C++ Build Tools, gh) | готово, всё на месте |
| 1 | `cargo test --workspace` | зелёный, exit 0 |
| 2 | Создан `victory.toml` с `game_dir = 'C:\GTA6_Files'`, проверен `victory config` | готово |
| 3 | `cargo build --release -p victory` | готово, 1 мин 33 с |
| 4 | Записаны две команды для PowerShell (ниже) | готово |
| 5 | Полная предкомпиляция шейдеров (`--warm all`, 12 потоков) | готово: 25 889 запрошено, 25 635 в кэше, 254 не переводятся, 5,1 с |
| 6 | Собрана диагностическая сборка `target\diag\victory.exe` `--features diagnostics` | готово, 1 мин 56 с |
| 6 | Прогрев кэша **этой** сборки (ключ кэша у неё свой) | готово: 25 635 переведено за 417,6 с |
| 7 | Тестовый запуск игры 1920x1080 + автоклавиши + скриншоты кадров | готово: дошла до экранов первичной настройки, 0 ошибок, ~27 fps |
| 8 | Прогон 2 (4 мин → 7 мин): автоклавиши до 95 с, захват мировых кадров | готово: **дошла до 3D-мира** (пролог, титры), 0.2–3.2 fps, 3509 draw/frame, падений нет |
| 9 | Ветка `perf-and-upstream-fixes`: аудит апстрима, перенос применимых правок, замер лимита дескрипторов | готово: 4 файла, +81/−24, `check` и `test` зелёные; предел D3D12 = 1 000 000 (см. §9) |
| 10 | Разведка: 2 692 форка через `gh api` + аудит пути копирования (copy-back, copies, textures, VRAM) | готово: кода не менял; готовых «многопоточных copy_back» нет, но есть идеи и ветки; замеры и выводы — §10 |
| 11 | Аудит двух коммитов-кандидатов, релизная сборка, прогон 3 с флагами разгрузки VRAM, замер FPS/VRAM | готово: коммиты не портируются (нет подсистемы/меняют шейдеры), релиз собран, прогон 655 с, мир с 126 с: медиана 1,7 fps, пик VRAM 96,7 % — §11 |
| 12 | Прогон 4: релиз на прогретой библиотеке пайплайнов, staging 128 МБ / 160 мс / catchup=1, замер FPS/VRAM/очереди и проверка чёрного кадра | готово: мир с 87 с, дошли до игровых катсцен; медиана 2,0 fps (уст. 1,8), пик 10,9 (в меню 30,2), очередь «просроченных» 12912→2392, VRAM пик 96,3 %, чёрных кадров не поймано — §12 |
| 13 | Порт prefetch-фикса copy-back (real69xaros, +37 строк), проверки, релизная сборка | готово: +55 строк в `shadow.rs` (свободная функция `prefetch` + вызовы в `find_pieces` и `copy_back_with`), `check` exit 0, полный `test` зелёный (2 первых прогона — известная флаки GPU/памяти, подтверждено базой), релиз 32,6 с — §13 |
| 14 | Разбор чёрного кадра в интро по коду и логам | готово: это не VRAM и не баг очистки — штатный чёрный кадр перехода/титров, растянутый в секунды штормом создания PSO (530–549 пайплайнов, кадр 3–7,6 с, ~1050 draw в `dropped`); с тёплой библиотекой в прогоне 4 не воспроизвёлся — §14 |
| 15 | A/B prefetch-фикса (3 прогона по 350 с), решение opt-in, итог ночи | готово: при равном контенте B(выкл) 5,9 fps против A2(вкл) 5,6; разброс между одинаковыми сборками того же порядка → выигрыш не подтверждён; фикс оставлен opt-in (`VICTORY_PREFETCH=1`, по умолчанию выкл.), релиз пересобран, check exit 0, полный test зелёный (3-я попытка; 1–2 — пре-существующие флаки warp-тестов) — §15 |
| 16 | Бэкап-репозиторий `4EKUSHKAq/victory-fork-3060ti` (private) + разведка fxceless/victory-fork, Marvisak/VIctory, xcl-dev/xcl и аудит наших 254 непереведённых шейдеров/HDR-флагов | готово: репозиторий создан, `myfork` добавлен, `perf-and-upstream-fixes` и `master` запушены; найдены конкретные кандидаты (bloom/movreld-фикс, #2268, NVIDIA wave64 #1249, BVH-путь, perf-серии), всё с пометкой влияния на кэш шейдеров; код не менялся — §16 |
| 17 | Ветка `visual-and-nvidia-fixes`: 3 субагента-разведчика, затем порт #2268 (user-data/вокселизация) и NVIDIA-слоя (#1176 + control-flow #1249); bloom f342dcfcfb не переносился (no-op) | готово: `check` после каждого шага exit 0, полный `test` зелёный, релиз 43,03 с, кэш прогрет 249,4 с (25 637 переводов), `--list`: **254→57** (исправлено 197, новых 0), DXIL compute +2 — §17 |
| 18 | Мониторинг 6 звёздных репозиториев + perf-батч без сброса кэша: #1237+#1245, #1227, host-часть #1224, VRAM-пара #1240+#1258, свежий #1269; #1178 отклонён | готово: `check` после каждого из 6 шагов exit 0, полный `test` зелёный (gpu 188 вместо 184), релиз 30,88 с, кэш шейдеров **не сброшен** (`0 translated now, 25637 in the cache already`, 4,7 с) — §18 |
| 19 | Публичный форк `codec-d/cool_fox`: remote+fetch, 3 субагента (лаунчер/pareto, перф, свет/шейдеры), выбор Пути А, ветка `coolfox-rtx`, сборка и прогрев под их транслятор | готово: `check --locked` exit 0, релиз 1 м 05 с (exe 9 979 904 Б), кэш `cache-pareto` прогрет 294,3 с (25 663 переведено, 226 не переводятся), `config` exit 0; запушено в `myfork` — §19 |
| 20 | Тестовые прогоны coolfox-rtx на 3060 Ti: авторский `.cmd` (Series S, 340 с) и диагностический Series X (320 с) с реальными нажатиями, скриншотами и nvidia-smi | готово: Series S = **чёрный 3D-экран** (two-plane compose), Series X = картинка + ambient; FPS медиана 2–3 у coolfox против **6** у нас, VRAM пик 7977/8192, 3 compute-шейдера дропнуты (2 водных у нас переведены); штатное закрытие не сработало — §20 |

## Окружение (проверено 2026-11-26)

| Компонент | Версия / состояние |
|---|---|
| Git | 2.56.1.windows.1 |
| Rust / cargo | 1.98.1 (тулчейн закреплён `rust-toolchain.toml`, автоматически доустановлен) |
| rustup | 1.29.3 |
| rust-analyzer | 1.98.1 — **не был установлен**, доставлен `rustup component add rust-analyzer` |
| C++ Build Tools | Visual Studio 2022 BuildTools, MSVC 14.44.35207 (`cl.exe`/`link.exe` найдены через vcvars64) |
| Windows SDK | 10.0.26100.0 (есть и более старые) |
| gh | 2.105.0, авторизован как `4EKUSHKAq` (scopes: repo, workflow, gist, read:org) |
| Процессор | Intel Core i5-10600, 6 ядер / 12 потоков |
| Память | 32 ГБ ОЗУ |
| Видеокарта | NVIDIA GeForce RTX 3060 Ti, 8 ГБ VRAM |

## Проверки

### 1. Тесты
`cargo test --workspace` — **успех, exit 0**. Все юнит- и интеграционные тесты зелёные
(victory-core 11, victory-formats 25, victory-gpu 184, victory-hle 100, victory-shader 216,
victory-loader 27 + 2 + 1, victory-pe 7, victory-pipeline 4, xtask 4, doc-tests 0).
Игнорируются только заявленные бенчмарки и тесты, которым нужны файлы игры/карта.

Замечание: первый прогон упал ровно на одном тесте — `xtask::policy::repository_passes_policy`
заметил мой собственный лог `cargo-test.log`, лежавший в корне репозитория (правило asset-policy:
в дереве не должно быть посторонних файлов). Лог перенесён в `%TEMP%`, повторный прогон — зелёный.
В репозиторий больше ничего не писалось.

### 2. Конфигурация
`victory.toml` создан копированием `victory.example.toml` (файл в `.gitignore`), в нём только
нужный путь, каталоги кэша/данных оставлены по умолчанию:

```
config file : victory.toml
game_dir    : C:\GTA6_Files
cache_dir   : C:\Users\4EKUSHKA\AppData\Local\victory\cache
data_dir    : C:\Users\4EKUSHKA\AppData\Local\victory\data
```

В `C:\GTA6_Files` найден исполняемый файл игры: `game_scarlett_master.exe`.
Проверка архива шейдеров: `victory shader-info C:\GTA6_Files\shaders.rpf` → **26 503 контейнера,
26 698 программ** (Ps 12 101, Ms 9 282, Cs 2 689, Vs 2 236, Es/Gs по 195). В остальных rpf
(`data`, `common`, `textures`, `script`, `appdata0`) контейнеров шейдеров — 0, то есть весь набор
лежит в `shaders.rpf`.

### 3. Релизная сборка
`cargo build --release -p victory` — успех за **1 мин 33 с**, ворнинги только про неиспользуемые
`placed_filler_twins` и поле `depth_written` (не мои правки).
Артефакт: `target\release\victory.exe`, 8 492 032 байт (2026-11-26 03:09).

### 4. Разминка кэша шейдеров (проверка команды, 30 с)
Команда из шага 4 прогнана с ограничителем `VICTORY_WARM_SECONDS=30`:
`11320 hashes asked, 11317 translated now, 0 in the cache, 3 do not translate; 30.0 s on 12 threads`.
То есть ~377 шейдеров/с на 12 потоках; полный набор 26 503 контейнера — примерно **70–90 с**.
Кэш: `%LOCALAPPDATA%\victory\cache\shaders`, после 30 с — 11 317 файлов, 218,5 МБ
(полный набор ожидается ~0,5 ГБ). 3 непереводимых программы — известное ограничение из README.

### 5. Полная предкомпиляция шейдеров
```
all hashes asked, 25889 found: 0 translated now, 25635 in the cache already, 254 do not translate; 5.1 s on 12 threads
```
То есть к этому моменту все 25 635 переводимых программ уже лежали в кэше
(`%LOCALAPPDATA%\victory\cache\shaders`, 25 635 файлов / 830 МБ), 254 — известное ограничение
из README. Повторный прогон полного набора занимает ~70–90 с; здесь он только подтвердил полноту.

### 6. Диагностическая сборка (нужна для автоклавиш и скриншотов)
`VICTORY_KEYS` и трассировка кадров компилируются только с фичей `diagnostics`, а признак этой
фичи входит в ключ кэша шейдеров (`crates/victory-shader/src/emit/mod.rs:202`). Поэтому:

```powershell
cargo build --profile diag -p victory --features diagnostics      # target\diag\victory.exe, 1 мин 56 с
$env:VICTORY_HOST_WAVE='32'; $env:VICTORY_VIEW_LIMIT='999999'
.\target\diag\victory.exe translate 'C:\GTA6_Files\shaders.rpf' --warm all --jobs 12   # 417,6 с, 25 635 переведено
```
Итог: в кэше теперь два набора под двумя ключами — 51 272 файла / 1 660 МБ. Оба запуска
(обычная и диагностическая сборка) кэш друг другу не портят, но и не делят.

### 7. Тестовый прогон игры
Запуск: `C:\GTA6_Port\run-test.ps1` (скрипт поднимает `target\diag\victory.exe run … --execute`
с флагами скорости из шага 4 плюс `VICTORY_HOST_WAVE`/`VICTORY_VIEW_LIMIT`, автоклавиши
`VICTORY_KEYS=25:Enter,30:Enter,35:Enter,40:Enter`, `VICTORY_CAPTURE=C:\GTA6_Port\test_frames`,
`VICTORY_CAPTURE_WORLD=1`; логи — `C:\GTA6_Port\run-logs\out.log` и `err.log`).
Съёмка окна — `C:\GTA6_Port\monitor.ps1`.

| Что | Результат |
|---|---|
| Игра поднялась | да: `Americas` в заголовке окна, окно 1920x1080 (на экране ужато до 1365x768) |
| Сколько шло | 03:29:32 → 03:31:47 (135 с), остановлено `Stop-Process` (не вылет) |
| Куда дошла | выбор учётной записи → субтитры → калибровка яркости → экран загрузки |
| До мира | нет: кадров с 500+ draw не было, максимум 16 draw/frame |
| FPS | 18 замеров: среднее **27,2**, min 6,9 (первая секунда загрузки), max 30,5 — упор в собственный лимит игры 30 fps в меню |
| Худший кадр | 145,6 мс (6,9 fps) в первые секунды, дальше 33,3 мс стабильно |
| VRAM | 1 725 МБ из 7 256 МБ бюджета (≈24 %), ОЗУ процесса ~3,3 ГБ |
| `crashes.log` | **нет файла** — падений не было |
| Логи | 0 строк ERROR; 29 × «pipeline creation method not implemented», 12 × повторная текстура, 5 × эмуляция инструкций CPU (`Clwb` и др.), 4 × чёрный кадр на загрузке (кадры 1–102) |
| Кэш за прогон | +5 файлов: 25 635 шейдеров были готовы заранее, то есть прогрев работает |
| Скриншоты | 171 PNG в `C:\GTA6_Port\test_frames` (1920x1080); fps нарисован в углу окна (видно на снимке `run-logs\window-120s.png`), в самих PNG его нет |

Отдельно замечено: на старте слой пишет
`host view heap capped at 1000000; views past it read a null slot count=1245184` — игра просит
1 245 184 дескриптора, а хост даёт 1 000 000. В мире это повторилось (см. §8): 245 184 вида
читаются как null. Правка — `PORTABLE_VIEWS`/`UNCAPPED_VIEWS` в `crates/victory-gpu/src/descriptor/host.rs`.



### 8. Прогон 2: дошли до мира (2026-11-26, 03:37:18 → 03:44:15)
Скрипт `C:\GTA6_Port\run-test2.ps1`: те же флаги, `VICTORY_KEYS=25,30,35,40,50,65,80,95:Enter`,
`VICTORY_CAPTURE_WORLD=1` + `VICTORY_CAPTURE_WORLD_ALSO=1,5,15,30` +
`VICTORY_CAPTURE_WORLD_EVERY=64` (реже читать кадр назад, чтобы не портить замер).
Логи: `C:\GTA6_Port\run-logs2`, кадры: `C:\GTA6_Port\test_frames2`.

> Важно: `VICTORY_CAPTURE_WORLD` проверяется только на «задано/не задано», а список кадров
> задаётся отдельной переменной `VICTORY_CAPTURE_WORLD_ALSO`. Запись `VICTORY_CAPTURE_WORLD=1,5,15,30`
> сама по себе дала бы только кадры 2, 6, 30 и 80 после первого кадра с 500+ draw.

| Что | Результат |
|---|---|
| Длительность | 417 с (7 мин); я продлил сверх 240 с, потому что мир начался только на 154-й секунде и на 240-й секунде на экране был лишь титр |
| Экран EULA | игра упёрлась в «Юридические условия и политики Rockstar» (нужна галочка второго пункта, не только Enter) — **обошёл ручной отправкой клавиш** (`Down`, `Enter` через `keybd_event`, скрипт `press.ps1`), после этого пошла загрузка |
| Дошла до мира | **да**: кадр 3785 — первый с 500+ draw (титр «В АЙС-СИТИ, ДВА ГОДА НАЗАД»), затем вступительные титры |
| draw/frame в мире | от 29 до **3509**, типично 700–1000 |
| FPS в мире | 47 замеров: **среднее 1,86**, медиана 2,0, **min 0,2**, max 3,2; медиана кадра 515 мс, худший кадр **5114 мс**, 199 «длинных кадров» |
| Где уходит время | GPU **0,0 мс/кадр** (простаивает): подготовка ~241 мс (копии 80 мс / 132 МБ, текстуры 85 мс / 76 МБ) + копирование назад 102 мс / 30 МБ |
| Брошенные draw | 240–937 за кадр (до 27 %) — «without a host pipeline»: ~950 пайплайнов создаются одновременно |
| VRAM | пик **6802 МБ** из ~7104 МБ бюджета (96 %), один раз «the card has no room for more», 206 повторных загрузок текстур |
| Кэш дескрипторов | `host view heap capped at 1000000; views past it read a null slot count=1245184` — 245 184 вида из 1 245 184 читаются как null |
| `crashes.log` | **нет файла** — падений не было |
| Ошибки | 0 строк ERROR; предупреждения: 206 × «текстура, выбранная по индексу, изменилась» (перезалив), 29 × нереализованный способ создания пайплайна, 5 × эмуляция инструкций CPU, 8 × чёрный кадр в мире, 4 × `host pipeline refused 0x80070057` (compute), по 1 × «descriptor не выровнен», «буфер-вью оставлен null», «шейдер берёт адрес отсутствующего буфера» |
| Артефакты | (1) кадры 3786–3983 периодически полностью чёрные, только субтитр; (2) в кадре 4233 надписи титров накладываются друг на друга (в 4105/4169 те же титры чистые) — похоже на неочищенный буфер/выпавшие проходы |
| Переводы шейдеров | 1212 переводов за прогон, но в кэш **не записалось ни одного файла** (0 файлов новее 03:37) — записи не долетают, если процесс убит |
| Кадры | 255 PNG, 57 «тяжёлых» (>200 КБ); мировые: 3786…4233 |

## Команды для пользователя (PowerShell, из корня `C:\GTA6_Port\victory-fork`)

### Предварительная компиляция шейдеров
```powershell
$env:VICTORY_HOST_WAVE='32'; $env:VICTORY_VIEW_LIMIT='999999'; .\target\release\victory.exe translate 'C:\GTA6_Files\shaders.rpf' --warm all --jobs 12
```

### Запуск игры в 1920x1080 с флагами скорости
```powershell
$env:VICTORY_DISPLAY_MODE='1920x1080'; $env:VICTORY_LEVELS_ON_DEMAND='1'; $env:VICTORY_ASYNC_SUBMIT='1'; $env:VICTORY_QUEUE2='1'; $env:VICTORY_INDIRECT_WINDOW='1'; $env:VICTORY_TEXTURE_NO_WAIT='1'; $env:VICTORY_MERGE_GAP='64'; $env:VICTORY_MERGE_RUNS='4096'; $env:VICTORY_STAGING_POOL='0'; $env:VICTORY_COPY_PLAIN='1'; $env:VICTORY_JOIN_PARTS='1'; $env:VICTORY_BACK_OWN_VIEW='1'; $env:VICTORY_INDIRECT_GATHER='1'; $env:VICTORY_CARD_REHASH_RUNS='1'; $env:VICTORY_HOST_WAVE='32'; $env:VICTORY_VIEW_LIMIT='999999'; .\target\release\victory.exe run 'C:\GTA6_Files\game_scarlett_master.exe' --execute
```
Для автоклавиш и скриншотов кадров нужен `.\target\diag\victory.exe` (сборка с `diagnostics`)
и её собственный прогрев: `VICTORY_KEYS` и трассировка кадров в обычной релизной сборке не
компилируются, а признак фичи входит в ключ кэша шейдеров.

### Почему именно так
- Флага `--cache` у команды `translate` нет: перевод **всегда** идёт в кэш запуска
  (`%LOCALAPPDATA%\victory\cache\shaders`). Предварительный прогрев — это `--warm all`.
  Именно поэтому в команде его нет.
- Переменной `VICTORY_WINDOW_MAX` в коде нет ни в одном виде. Размер задаётся тем, какой
  экран слой сообщает игре: `VICTORY_DISPLAY_MODE=WxH` (`crates/victory-gpu/src/dxgi.rs:222`).
  Окно создаётся игрой по размеру её render target и лишь ужимается под экран
  (`crates/victory-hle/src/modules/lifecycle.rs:113`). При `1920x1080` игра рендерит 1080p —
  это ещё и снимает часть давления на 8 ГБ VRAM.
- `VICTORY_HOST_WAVE=32` и `VICTORY_VIEW_LIMIT=999999` обязательны при прогреве: ключ кэша
  шейдеров включает эти два факта хоста (`crates/victory-shader/src/emit/mod.rs:199`), а у NVIDIA
  волна 32 линии и куча дескрипторов урезана до 1 000 000 слотов
  (`crates/victory-gpu/src/descriptor/host.rs:795`). Без них прогретые шейдеры в запуске не найдутся.
- Набор флагов скорости — тот самый «switch set of run 253» из README (строка 45) плюс
  режим 1080p. Комбинация `VICTORY_INDIRECT_WINDOW` + `VICTORY_CARD_FETCH_INFLIGHT` запрещена
  README, `CARD_FETCH_INFLIGHT` в команде нет.
- `--jobs 12` — по числу логических потоков i5-10600 (по умолчанию код берёт всего 4).

## Известные ограничения / что не проверено
- Мир проверен только на вступительной сцене (титр + титры пролога). Собственно город,
  геймплей, машины и толпа не проверялись.
- Экран EULA требует галочки, а не только Enter: автоматика на одних Enter его не проходит,
  нужны `Down` + `Enter` (или X). В следующий автопрогон это надо зашить в `VICTORY_KEYS`.
- В меню fps упирается в собственный лимит игры 30 fps; в мире — 0,2–3,2 fps при 96 % VRAM.
- Главный тормоз — не карта (GPU 0,0 мс/кадр), а подготовка: 80 мс копий (132 МБ/кадр),
  85 мс текстур (76 МБ/кадр) и 102 мс обратного копирования (30 МБ/кадр).
- Тени/отражённое освещение всё ещё чёрные — статус форка из README.
- Размер окна задаётся `VICTORY_DISPLAY_MODE`, а не `VICTORY_WINDOW_MAX` (переменной нет).
- 1 прогон из истории README терял устройство при `VICTORY_TEXTURE_BACK=1` — этот флаг не включён.
- `VICTORY_MERGE_GAP`/`MERGE_RUNS` — не точная эмуляция (см. предупреждение в README и
  комментарий у `merge_gap`), поэтому при артефактах картинки их снимают первыми.
- Переводы шейдеров, сделанные во время игры, в кэш на диске не попадают, если процесс убит
  (1212 переводов за прогон 2 — 0 файлов в кэше). Нужен чистый выход игры, иначе каждый
  запуск переводит их заново.
- Переменные окружения из команды действуют только в текущем окне PowerShell. Для чистого
  старта: `Remove-Item Env:VICTORY_*`.

## §9. Аудит апстрима на ветке `perf-and-upstream-fixes` (2026-11-26)

Ветка создана от `check-env-and-build`. Скачаны диффы (`C:\GTA6_Port\upstream\`):
PR #2850, #2851, #2853, #2859, #2865 и issue #2826/#2847. **Ни один дифф не лёг патчем
(`git apply --check` — конфликт у всех пяти):** форк hyphennn разошёлся с master настолько,
что у половины правок просто нет того кода, который они правят.

| Апстрим | Что делает | Что у нас |
|---|---|---|
| Issue #2826 (copy-back по страницам в несколько потоков) | «главный рычаг FPS» | **готового патча нет** — в апстриме issue закрыта триажем: «No open PR spreads this loop over threads… PRs are open to everyone, please open one». У нас и функции `copy_back_marks` нет: копирование идёт через `Shadow::copy_back_with(…, found)`, а куски ищутся на нескольких потоках (`shadow.rs:1589`) — то есть у нас уже иначе, чем в master |
| PR #2859 (малые мипы: `Layout::bytes_from`, `first_resident_mip`) | не читать всю текстуру, когда пришли только малые мипы | **не переносил**: у нас стриминг сделан иначе — `copy_mapped` читает весь размер, подставляя нули на незамапленных страницах, плюс повтор раз в 30 кадров (`texture.rs:1089`). `bytes_from` без переписывания этого пути не используется |
| PR #2853 (кэш пайплайнов + residency в шейдерах) | сохранять кэш при неудачной записи; отдавать только резидентные BVH-узлы | **частично уже есть**: `VICTORY_WARM_SECONDS` есть, `save()` новее апстримного. Перенёс возврат `USED_CHANGED` при ошибке записи (повтор сохранения). Шейдерную часть (`lower.rs`, `lower/bvh.rs`) не переносил: у нас другой `memory()`, других подписей `node_dwords`/`raw_load` нет |
| PR #2850 (CBV на попадании в кэш `fresh_tables`) | обновлять constant buffer view, а не только SRV/UAV | **перенёс** — в `draw.rs` убран фильтр `u.kind != ResourceType::Cbv` |
| PR #2851 (null SRV → `alias_source`) | не разыменовывать null | **уже есть**: нашего `alias_source` нет, его роль играет `alias_of`, и он начинается с `this.is_null()` |
| PR #2865 (null-защиты) | heap null в `Resource::data`, `resource != 0` в `readable`, `planes.first()` | **перенёс** `data()` (null heap → null ptr) и `resource != 0`; `host_resource`/`host_copy` уже проверяют null; `planes.first()` — такого поля у нас нет |

### Что изменено (4 файла, +81/−24)
- `crates/victory-gpu/src/resource.rs` — `Resource::data()`: нулевая куча → нулевой адрес (из #2865).
- `crates/victory-gpu/src/draw.rs` — CBV в кэше `fresh_tables` теперь обновляется (из #2850);
  `readable` не трогает нулевой `View::Shader` (из #2865).
- `crates/victory-gpu/src/pipeline/library.rs` — при неудачной записи флаг `USED_CHANGED`
  возвращается, сохранение повторится (из #2853).
- `crates/victory-gpu/src/descriptor/host.rs` — из `probe_view_limit` выделен `takes_views(count)`,
  в предупреждение о клампе добавлено `fitted`, добавлен замерный тест (см. ниже).

**Ключ кэша шейдеров эти правки не меняют** — прогревать заново не нужно (ни `victory-shader`,
ни подпись транслятора не тронуты).

### Дескрипторы: 1 000 000 — это предел железа, а не наша настройка
`probe_view_limit()` уже пробует `UNCAPPED_VIEWS` и откатывается, если не вышло — включать
руками нечего. Замер на этой машине (`cargo test --release -p victory-gpu -- --ignored
view_heap_size --nocapture`):

```
1000000 views: true
1048576 views: false
1245184 views: false
1500000 views: false
2097152 views: false
```
То есть RTX 3060 Ti берёт ровно 1 000 000 дескрипторов — документированный предел D3D12
(`D3D12_MAX_SHADER_VISIBLE_DESCRIPTOR_HEAP_SIZE_TIER_2`). Запрос игры 1 245 184 в одну кучу
не помещается ни при какой настройке — нужен расклад вида по двум кучам, это отдельная работа.
Заодно видно, что кламп режет сильнее, чем нужно: из 1 000 000 слотов 2^18 (`DYNAMIC`) отданы
под таблицы, поэтому игра получает `fitted = 737 855` слотов. Комментарий в коде говорит, что
реально игра доходит до ~720 000, так что, скорее всего, потерь нет — в предупреждение добавлено
`fitted`, следующий прогон это покажет.

### Проверки
- `cargo check --workspace` — **ок** (только известный ворнинг `placed_filler_twins`).
- `cargo test --workspace` — **ок, 0 падений**: ядро 11, форматы 25, gpu 184 (+1 новый ignored),
  hle 100, loader 27+2+1, pe 7, pipeline 4, shader 216, xtask 4 (asset-policy зелёный).

## Следующий шаг (по правилу «эстафеты»)

**Прогон 3 сделан (§11), ждём решение по §11.6.** Код по-прежнему не менялся, ветка
`perf-and-upstream-fixes`, `master` не тронут. Ближайшие шаги, по приоритету:
1. Повторить релизный прогон подряд: библиотека пайплайнов и шейдерный кэш уже на диске — второй
   запуск отделит «холодный старт» (3 227 PSO, 618 «делается» на кадр) от реального потолка.
2. Вернуть staging `VICTORY_TEXTURE_STAGE_MB=128…192` / `_MS=160…250` (или `_CATCHUP=1`): очередь
   16 782 текстуры не рассасывается на 64 МБ/80 мс; сравнить fps, «passed over unmapped» и VRAM.
3. Взять prefetch-фикс copy-back (real69xaros, +37 строк) — единственная дешёвая правка, бьющая в
   71 мс «copying back»; после порта — обязательные `cargo check --workspace` + `cargo test`.
4. Чёрные кадры: снять `VICTORY_CAPTURE_FRAMES` диагностической сборкой вокруг эпизода (в прогоне 3
   — t≈504 с) и смотреть `frame-<n>-ops.txt`; кандидаты из форков — `bodnjenie14` (Bink),
   `NitricAcidDev` (intro/overlay), но это ручной перенос.
5. mip-drop для 8 ГБ — только как отдельная задача «подсистема mip-drop» (§11.1), не порт 85 строк.
6. `HANDOFF.md` держать обновлённым; одна ступень = один шаг с отчётом.

## Файлы этого шага (вне репозитория)
- `C:\GTA6_Port\upstream\` — скачанные `pr2850.diff`, `pr2851.diff`, `pr2853.diff`, `pr2859.diff`,
  `pr2865.diff`, `issue2826.txt`, `issue2847.txt` (для аудита);
- `C:\GTA6_Port\run-test.ps1` / `run-test2.ps1` — запуск игры с флагами, автоклавишами и съёмкой кадров;
- `C:\GTA6_Port\press.ps1` — ручная отправка клавиш (`keybd_event`), чтобы обойти экран EULA;
- `C:\GTA6_Port\monitor.ps1` — заголовок окна (там fps) и снимок окна с экрана;
- `C:\GTA6_Port\run-logs\`, `run-logs2\` — `out.log`, `err.log`, снимки окна;
- `C:\GTA6_Port\test_frames\` (171 PNG), `test_frames2\` (255 PNG) — кадры прогонов 1 и 2.

## §10. Разведка форков и аудит копирования (2026-11-26, кода не менял)

Цель шага: найти готовые решения для −180 мс/кадр (copy-back 102 мс + copies 80 мс), давления
на 8 ГБ VRAM (пик 96 %, 206 перезаливов) и артефактов (чёрные кадры, наложение титров).
Итог коротко: единого патча «многопоточный `Shadow::copy_back_with`» нет ни в апстриме, ни в
форках, **но отдельные готовые компоненты есть по всем трём темам** (таблица ниже), включая
именно многопоточный copy-back; аудит показал, что 102 + 80 мс из §8 — это **худший** кадр
диагностической сборки, а не норма.

### 10.1. Форки: что реально есть (2 043 у Marvisak + 869 у fxceless = 2 696)

Метод: GraphQL-скан **всех** форков (27 запросов по 100, ~4,4 пункта лимита) — ветки и HEAD
default-ветки каждого, сравнение с 731 коммитом апстрима (`Marvisak/main` 691 + `fxceless/main`
253); затем `commits/{sha}` с файлами только у разошедшихся. Первичный проход через
`forks?sort=newest` оказался ловушкой: этот sort — по времени **создания** форка, поэтому «20
самых свежих» выглядели копиями. По фактическому `pushed_at` в топе-20 — 7 форков с реальной
работой, и полагаться на `pushed_at` всё равно нельзя (у нетронутого форка он равен моменту
создания).

| Что показал скан | Число |
|---|---|
| Всего живых форков (4 — 404/удалены) | 2 692 |
| Чистые копии (одна ветка) | 1 644 |
| Две ветки | 403 |
| >2 веток (своя работа) | 645 |
| Свой HEAD в default-ветке (`ahead_by > 0`) | 222 (у 13 из них ≥100) |
| Из 222: есть совпадения по A (copy-back/readback) | 67 |
| … по B (VRAM/текстуры/стриминг) | 131 |
| … по C (чёрные кадры/титры) | 59 |
| хотя бы одна из трёх тем | 138 |

Распределение `ahead_by` у 222: 1–4 — 88, 5–19 — 57, 20–49 — 29, 50+ — 48.

Таблица — только самое релевантное из найденного
(A — copy-back/readback, B — VRAM/текстуры, C — чёрные кадры/титры):

| Форк | ahead | A/B/C | Что там (проверено: сообщение + файлы) |
|---|---|---|---|
| [gabztoo/LumeGX](https://github.com/gabztoo/LumeGX) | 73 | **A** | `58fc51b6` «snapshots and copy-back of shader-written ranges **on several threads**» → shadow.rs **+399/−47** — прямой ответ на вопрос о многопоточном copy-back |
| [real69xaros/victory-laptop](https://github.com/real69xaros/victory-laptop) | 78 | **A** | `8e08d08f` «one **PrefetchVirtualMemory** call before a copy-back compares pages — 49 745 faults/frame was **145 ms of a 263 ms** frame» → shadow.rs **+37** |
| [cxchhh/victory-gta6](https://github.com/cxchhh/victory-gta6) | 83 | **A** | `93e26e93` «title memory read/written by fault-tolerant copy **instead of kernel calls** (`VICTORY_FAST_COPY`)» → trycopy.rs +123, inplace.rs +49, host.rs +42 |
| [Steenax6/VIctory-NVID](https://github.com/Steenax6/VIctory-NVID) | 45 | **A** | `3a0b70b8` «runs of a copy-back that lie close together go back **in one write** (240 000 writes/frame under Wine)» → shadow.rs +70 |
| [zhaibingye/VIctory](https://github.com/zhaibingye/VIctory) | 82 | A | `f270bd63` «vectorize shadow writeback merging» → shadow.rs +41/−18 |
| [wbkepler/VIctory](https://github.com/wbkepler/VIctory/tree/gpu/lazy-copy-back) | 81 | A, B, C | lazy copy-back (`VICTORY_LAZY_COPY_BACK=1`, shadow.rs +517, watch.rs +148), `copy-back-block-compare`, `copy-back-gpu-bytes-only`, `skip-clean-texture-uploads` (наши 206 перезаливов), `title-memory-bitmap-in-vram`, `default-vram-shadow-buffers` |
| [sirrus98/VIctory](https://github.com/sirrus98/VIctory/tree/perf/up-hot-vram) | 9 | A, B | `resident.rs` +832: title-буферы в VRAM, назад только изменённые dword'ы, бюджет 60/80 % (DRAFT #2734) |
| [0xkn0wh3r3/VIctory](https://github.com/0xkn0wh3r3/VIctory/tree/gpu/parallel-refresh) | 68 веток | A | `copy_pool.rs` +233 — refresh'и одной посылки копируются на нескольких потоках (+тест) |
| [ExoCalibra/victory-exo](https://github.com/ExoCalibra/victory-exo) | 92 | A | «Shadow::refresh_many (#1068)», быстрый путь чтения `shadow::BUFFERS`, batched `defer_refreshes` |
| [triippywhale/victory-fork](https://github.com/triippywhale/victory-fork) | 100 | A | «relay_merge и card_merge пишут только изменённые run'ы», «stretch >8 run'ов пишется целиком» |
| [Bezouro/VIctory](https://github.com/Bezouro/VIctory) | 100 | A | «publish color texture readbacks through fenced buffer shadow» → list/readback.rs +249 (общий readback-буфер) |
| [AntoninMichelq/VIctory-forkAMD](https://github.com/AntoninMichelq/VIctory-forkAMD) | ≥100 | A, B, C | `VICTORY_WIDE_TEXTURES` (детелинг на потоках), `VICTORY_VRAM_HYSTERESIS`, `VICTORY_STAGING_POOL_MB`, эвикция старых host-копий, staging на своём потоке |
| [ESAT586/VIctory](https://github.com/ESAT586/VIctory/tree/gpu/auto-mip-drop) | ветки | B | `mip_drop=1` по умолчанию на **≤8 ГиБ VRAM** (наш кейс), + авто на `E_OUTOFMEMORY` → host.rs +14, resource.rs +71/−6 |
| [brololikeptputtingmypassword/victory-fork-BESTegolol](https://github.com/brololikeptputtingmypassword/victory-fork-BESTegolol) | 52 | B | «Placed textures: host copies start at first mip shown» (`VICTORY_MIP_COPIES=1`), «stale texture slots keep what they show» |
| [mercuryVM/VIctory](https://github.com/mercuryVM/VIctory) | 71 | B | интегратор апстрим-PR (#1825, #1765, #1913 …) — удобно как «сборка готового» |
| [itsonlysimbo/VIctory](https://github.com/itsonlysimbo/VIctory/commit/b12857f8) | 100 | **C** | тот же коммит, что у techselect, но полный: «Fix GPU hangs, memory leak, mesh shader exports; **render intro credit text**» → host.rs +142, draw.rs +44, lower.rs +129, xmem.rs +12 |
| [NitricAcidDev/VIctory-Omega-Fork](https://github.com/NitricAcidDev/VIctory-Omega-Fork) | 75 | **C** | «**Overlays and movies that keep changing are shown as they are; intro uploads by default**» |
| [bodnjenie14/VIctory](https://github.com/bodnjenie14/VIctory) | 17 | **C** | «keep actively changing view-memory textures hot to **fix Bink video flickering**» — наш чёрный кадр на видео/титрах |
| [HoLa-Lee/victory-wave-wide-barriers](https://github.com/HoLa-Lee/victory-wave-wide-barriers) | 92 | A/B/C | замеры под наш профиль: «copy-back 92 ms», «2–3 мс на мегабайт», «64 MB staging → катсцена целиком»; наблюдение: **без копий картинка — чёрный кадр** (копии «несущие») |

Апстрим-трекер (свежее, OPEN): #1215 (copy-back через `NtWriteVirtualMemory`, 1.5 с → 0.13 с),
#2242/#2280/#2409 (copy-back только изменённых байт / блоками / без no-op), #2818 (общие чанки
readback), #1124 (батчинг записи copy-back), #2389 (стриминговые текстуры занимают VRAM только
под держимые уровни, «~5 ГБ меньше»), #2222 (DRAFT: upload-копии получают долю бюджета VRAM),
#2146/#1272 (mip-drop), #2837/#1254/#2666 (диагностика чёрного кадра), #319/#1799/#1076 (flip-wait).
И главное: **issue #407 «one address space for title memory»** (OPEN, `triage: has patch`) — арена
над CPU-видимой кучей вместо копирования: это прямая замена всему readback, а не оптимизация.

**Чего нет:** единого патча «переписать `Shadow::copy_back_with` на N потоков» нет — параллелизм
лежит внутри `shadow.rs`/`watch.rs`/`host.rs` (+ новые `copy_pool.rs`/`trycopy.rs`), `rayon` не
используется; «фикса чёрных кадров» одной строкой нет — есть диагностика и набор компонентных
фиксов (Bink/overlay/intro); «не накладывать титры» именно как фикс нет — есть рендер текста
титров (`itsonlysimbo`/`techselect`). Все патчи — против апстримных файлов; у нас своя линия (нет
`copy_pool.rs`/`resident.rs`/`xmem.rs`/`trycopy.rs`, зато есть `relay`, `wide_back`, `queue2`,
`plaincopy`, `backreads`), поэтому переносить придётся руками, как в §9.

**Первыми смотреть (по соотношению «эффект/размер»):**
1. `real69xaros` — `PrefetchVirtualMemory` перед сравнением copy-back: **+37 строк**, а по их замеру
   из кадра уходило 145 мс из 263 (49 745 page fault'ов). У нас в худшем кадре `copystat` показывает
   **64 727 page fault'ов** — профиль тот же, значит это наш самый дешёвый кандидат.
2. `Steenax6` — «близкие run'ы — одной записью ядра» (+70 строк): у нас запись идёт по run'ам через
   `NtWriteVirtualMemory`, по одному вызову на run.
3. `gabztoo` — многопоточные snapshots/copy-back (+399 строк): это ровно наш вопрос «можно ли на
   12 потоков», уже написанный; брать как образец, адаптируя к нашему `relay`/`wide_back`.
4. `ESAT586` — `mip_drop` по умолчанию на ≤8 ГиБ (наш случай), `brololikept...` — `VICTORY_MIP_COPIES`;
   `AntoninMichelq` — `VICTORY_WIDE_TEXTURES`/staging-пул/гистерезис VRAM.
5. `itsonlysimbo` — титры (+142 host/+44 draw/+129 lower), `NitricAcidDev` — overlay/intro,
   `bodnjenie14` — Bink-мерцание; `HoLa-Lee` — замеры и объяснение чёрных кадров (копии «несущие»).

### 10.2. Аудит нашего кода: откуда на самом деле берутся 102 + 80 + 85 мс

Пересчёт по 48 «мировым» кадрам из `run-logs2\err.log` (кадр ≥ 200 мс; это **диагностическая**
сборка с трассировкой кадров, она завышает):

| Что | Медиана | Среднее | Худший кадр | Доля медианного кадра |
|---|---|---|---|---|
| кадр целиком | 506 мс | 883 | 5 114 | — |
| preparing | 98 мс | 141 | 1 565 | 19 % |
| — copies | **41,8 мс** / 63 МБ | 76 / 108 | 1 496 / 2 152 | 8 % |
| — textures | **14,3 мс** / 20 МБ | 30 / 39 | 209 / 287 | 3 % |
| copying back | **32,2 мс** / 7,6 МБ | 37,5 / 9,6 | 191 / 29,7 | 6 % |
| GPU | 0 | 7,4 | 146 | 0 |
| **проход по куче (heap buffers)** | **182 мс** | 330 | **3 297** | **36 %** |

То есть «102 мс + 80 мс» — это один худший кадр, а не норма: в типичном кадре всё вместе ~88 мс.
И почти треть кадра уходит на **проход по куче** (`heap buffers ... passes ... ms`), которого в §8
не было видно: это `State::heap_buffers`/`heap_textures` (`draw.rs:1531`, `draw.rs:1842`) —
стриминг текстур, выбираемых по индексу: бюджет **192 МБ/250 мс на кадр**, а в «догоняющем»
режиме ×3 (576 МБ / 750 мс), при 13 173 ждущих слотах (в логе — «heap buffers 1.0 passes 815.6 ms»).

#### 10.2.1. copies (80 мс в худшем кадре)
Это **не memcpy 132 МБ**. Разбор того же кадра (строка `copystat`): `moves 24.9 ms` (реальный
`copy_mapped`), `watches 50.2 ms` — **1 439 вызовов `GetWriteWatch`** на 2 773 МБ проверенной
памяти, ~35 мкс на вызов, все под **одним** мьютексом `JOURNAL` (`watch.rs:394`). То есть в
«copies» доминирует не копирование, а вопрос ядру «что изменилось»; сам перенос уже ~5 ГБ/с
(132 МБ за ~25 мс). Путь: `Batch::update` (`host.rs:7340`) → `Shadow::update_free` → `copy` →
`copy_mapped` (`host.rs:7445`, `ReadProcessMemory`); «не копировать всё» — `watch::writes`
(`watch.rs:394`), батчинг — `ask_ahead` (`watch.rs:98`, склейка диапазонов ближе 8 МиБ) и
`ask_once` (по умолчанию включён).

#### 10.2.2. textures (85 мс в худшем кадре)
`Staging::refresh` (`texture.rs:1011`), строго по одной текстуре в шаге `Step::Texture`
(`host.rs:7145`). В том же кадре: **78 МБ хэшировано `blake3` (`texture.rs:1106`), 74 МБ
«изменилось»** → детилинг 76 МБ + 237 записей об отправке (203 отложено бюджетом). Хэш и
detile — однопоточные, порядок «заголовок → хэш → detile → запись в лист».

#### 10.2.3. copying back (102 мс)
Это время **всего лендинга relay-потока** (`host.rs:4933`), а не работа потока, готовящего кадр.
По умолчанию `VICTORY_RELAY=2` (stage 3): copy-back идёт на своём потоке `victory-relay`
(`host.rs:3318`), а `present` его не ждёт (`late_stage`, `host.rs:2053`). В логе другие потоки
ждут карту **10–90 мс/кадр** («submissions sent ahead … ms waiting for the card»), типично 15–25.
То есть 102 мс в основном **скрыты** за подготовкой; реальный простой из-за них — десятки мс в
редких кадрах. Внутри лендинга сравнение кусков уже многопоточное (`find_all`/`find_pieces`,
`shadow.rs:1590`), а **слияние** (то самое «копирование страниц») — один поток: `relay_merge`
(`shadow.rs:3011`) на каждый различающийся 64-КиБ кусок под замком снапшота; запись — `read_into`
(`host.rs:7566`) + `write_mapped` (`host.rs:7612`). Объём: 7,6 МБ в медиане, 30 МБ в худшем кадре.

#### 10.2.4. Что уже многопоточно и сколько потоков реально
- Уже параллельно: снапшоты (`writing` → `take_snapshots` → `copy_jobs`, `shadow.rs:1543`),
  сравнение (`find_pieces`/`differing`, `shadow.rs:3162`), и copy-back как таковой — на relay-потоке.
  Но пул `wide` — это **3 помощника** (`available_parallelism()/4`, `shadow.rs:1458`) + вызывающий
  поток = 4 потока, а не 12.
- Последовательно: refresh-копии (title→copy) в `Batch::flush` (`host.rs:7388`) — есть готовый
  выключатель `VICTORY_WIDE_REFRESH=1` (`host.rs:7175`, **по умолчанию выключен**, в run 112 был
  измерен хуже: 52 → 68 мс); слияние copy-back; детилинг/хэш текстур.

#### 10.2.5. Можно ли безопасно распараллелить и что реально даст
- **Refresh-копии на 12 потоков** — по состоянию данных безопасно (у каждой копии свои `pending`,
  карта `copied`, снапшот; общие — журнал, guards, счётчики — под замками/атомиками), но **не
  даст 80→20**: 50 из 80 мс — это `GetWriteWatch` под одним `JOURNAL`, они сериализуются сами.
  Потолок ~5–15 мс в медиане; шкалирование упрётся в мьютекс и в само ядро. Ветка 0xkn0wh3r3
  (`gpu/parallel-refresh`) делает ровно это — её и стоит взять как образец, но мерить.
- **Текстуры** — вот здесь параллелизм почти линейный: каждая `Staging` пишет свой буфер,
  хэш и detile независимы, поток-локальные скрэтчи. 14–85 мс → ~5–30 мс на 4 потоках (экономия
  ~8 мс в медиане и ~55 в худшем кадре). Аналоги у AntoninMichelq (`VICTORY_WIDE_TEXTURES`).
  Риск: важен порядок записей в лист (upload'ы должны попасть в лист в нужном порядке) —
  параллелить только хэш+detile, записи делать в исходном порядке; дедуп по `Arc` (Batch уже
  держит `HashSet`).
- **Слияние copy-back внутри одной копии** параллелить нельзя: замок снапшота + семантика «каждый
  байт один раз» + `watch::writing` на страницах title-памяти. Между **разными** копиями — можно,
  но у них могут пересекаться диапазоны title-памяти (разные host-копии одного буфера), и порядок
  записи тогда перестаёт быть детерминированным → риск «то чужой байт, то свой». При объёме
  7,6 МБ и скрытости этого времени — не стоит.
- Итого честно: чистый параллелизм даёт ~20–30 мс в медианном кадре (506 → ~480, +5 % fps) и до
  ~100 мс в худших. **Не 180.** Чтобы взять 180+, надо трогать проход по куче (182 мс медиана,
  3 297 мс максимум) и бюджет стриминга текстур, а не copy-back.

#### 10.2.6. VRAM (пик 96 %)
На конец прогона: бюджет карты 7 098 МБ, **в использовании 6 629**, из них host-копии под запись
карты 2 944 МБ (779 штук), host-копии текстур 1 835 МБ (8 157 штук), staging 404 МБ, host-копии
title-памяти 3 242 МБ; ОЗУ процесса 20 074 МБ приватных (14 266 в памяти, свободно 4 600).
Проверка `room()` (`resource.rs:1416`) пускает дальше только до **90 % бюджета = 6 388 МБ**, а
использовано 6 629 → **«места нет» постоянно**: в логе «passes that made none for lack of room:
1 520», один раз «the card has no room for more». 206 перезаливов — это `recheck` текстур,
выбираемых по индексу (`draw.rs:1976`: «a texture picked by index changed after its upload…»),
то есть симптом стриминга, а не отдельный баг. Ручки без кода: `VICTORY_TEXTURE_BUDGET_MB`
(потолок host-копий текстур), `VICTORY_CARD_ROOM_FOR_TARGETS_MB` (резерв под render target'ы,
сейчас 3 072 МБ), `VICTORY_TEXTURE_STAGE_MB`/`_MS` (бюджет прохода), `VICTORY_TEXTURE_CATCHUP=0`,
`VICTORY_REUPLOAD_WHEN_FULL=0`. По форкам под наш случай есть `mip_drop` по умолчанию для 8 ГиБ
(ESAT586) и эвикция/гистерезис (AntoninMichelq).

### 10.3. Варианты (нужно решение, кода пока не писал)

**A. Дешёвый замер без кода** (1 прогон релизной сборки, не diag; клавиши EULA — `Down`+`Enter`):
снять те же медианы в релизе (diagnostics завышает: `backreads::touched` на каждый read/write +
таймеры) и прогнать пары флагов: `VICTORY_WIDE_REFRESH=1`; `VICTORY_TEXTURE_STAGE_MB=64`+
`_MS=80`+`_CATCHUP=0`; `VICTORY_REUPLOAD_WHEN_FULL=0`; `VICTORY_TEXTURE_BUDGET_MB=2500`.
Это может дать больше, чем любая правка кода, и покажет реальные числа.

**B. Малые порты (1–2 дня, самый выгодный порядок)**: сначала три вещи, которые прямо бьют в
измеренные цифры: (1) `PrefetchVirtualMemory` перед сравнением copy-back (real69xaros, +37 строк —
у нас в худшем кадре 64 727 page fault'ов, тот же профиль, что их 49 745/145 мс); (2) склейка близких
run'ов copy-back в одну запись ядра (Steenax6, +70) и/или векторная склейка (zhaibingye, +41);
(3) параллельный refresh по образцу `copy_pool.rs` (0xkn0wh3r3) — только после замера A, потому что
run 112 показал хуже. Затем: параллельный хэш+detile текстур (`VICTORY_WIDE_TEXTURES`,
AntoninMichelq); пул `wide` 3 → 5; `mip_drop` для ≤8 ГиБ (ESAT586) / `VICTORY_MIP_COPIES`;
кэп бюджета staging по VRAM. Ожидаемо −30–60 мс медианы, −150–300 мс в догоняющих кадрах.

**C. Крупное (убирает 102 мс целиком)**: `VICTORY_LAZY_COPY_BACK`/marked copy-back (wbkepler,
готовый набор из 46 коммитов; вариант «только изменённые байты + блоками» — PR #2242/#2280/#2409)
или `resident.rs`/hot-VRAM (sirrus98, DRAFT #2734): шейдеры пишут прямо в VRAM-резидентную копию,
назад только изменённые dword'ы. Максимальный выигрыш и максимальный риск (артефакты/зависания).
Радикальный вариант — апстрим issue #407 (арена над CPU-видимой кучей, `triage: has patch`):
заменяет readback как механизм.

**D. Артефакты (отдельно, не смешивать с FPS)**: титры — `itsonlysimbo`/`techselect` (+142 host,
+44 draw, +129 lower: рендер текста титров, и это же чинит mesh-экспорты и утечку); overlay/intro —
`NitricAcidDev`; Bink-мерцание — `bodnjenie14`; flip-wait — `xmvo`. Чёрные кадры в прологе могут быть
не «багом отрисовки», а следствием недогруженных текстур (наблюдение `HoLa-Lee`: без копий кадр
чёрный) — тогда лечатся стримингом, а не шейдерами.

### 10.4. Риски (data race и не только)
- Refresh параллельно: гонок по данным нет (состояние на копию), но `JOURNAL`/guards сериализуют;
  главный риск — не гонка, а **масштабирование**: run 112 уже показал хуже; нужен замер.
- Текстуры параллельно: нельзя менять порядок записей в лист; дедуп по `Arc`; при
  `VICTORY_TEXTURE_NO_WAIT=1` чтение через `copy_mapped` (kernel) — безопасно, но 4 потока
  упрутся в память/ядро.
- Copy-back между копиями: пересекающиеся диапазоны title-памяти → недетерминированный «последний
  писатель»; текущая последовательная семантика это гарантирует. Не трогать без группировки по
  непересекающимся диапазонам.
- VRAM-ручки: меньше staging/бюджет → больше «пустых» текстур и чёрных поверхностей (сейчас и так
  13 173 слота ждут); `REUPLOAD_WHEN_FULL=0` → текстура, залитая после первой отправки, останется
  пустой (это уже чинили в run 179).
- `mip_drop=1` — потеря детализации; на 8 ГиБ это компромисс, который апстрим-форк считает
  правильным по умолчанию.
- Всё в ветке `perf-and-upstream-fixes`; `master` не трогается; после любой правки —
  `cargo check --workspace` + `cargo test --workspace` (AGENTS.md).

### 10.5. Проверки этого шага
Кода не менял. `git status` — 4 изменённых файла из §9 + `AGENTS.md`/`HANDOFF.md` (untracked).
Разведка: полный GraphQL-скан 2 692 форков (27 батчей, ~4,4 пункта лимита) + ~150 REST-вызовов
(коммиты и файлы ~30 веток/коммитов), rate limit не превышен; ключевые находки проверены через
`commits/{sha}` (файлы и размеры совпадают с заявленными).
Замеры §10.2 — из `C:\GTA6_Port\run-logs2\err.log` (прогон 2, диагностическая сборка).

## §11. Релизный замер с разгрузкой VRAM (2026-11-26, код не менялся)

Решение пользователя: путь стабилизации картинки и разгрузки 8 ГБ VRAM (вариант A + точечные
визуальные фиксы), без переработки copy-back. Шаг: проверить два маленьких коммита, собрать
релиз, прогнать с флагами разгрузки VRAM, замерить реальный FPS и VRAM.

### 11.1. Два коммита-кандидата: оба не портируются (код не менялся)

Скачаны через `gh api ... -H 'Accept: application/vnd.github.diff'` в
`C:\GTA6_Port\upstream\esat-mipdrop.diff` (5,9 КБ) и `techselect-credits.diff` (57,4 КБ), разобраны.

| Коммит | Что в нём | Почему не переносим |
|---|---|---|
| [ESAT586 da1f609f](https://github.com/ESAT586/VIctory/commit/da1f609f541f8d2c65fcddf7efeea04bb43ccf27) «mip_drop=1 на ≤8 ГиБ VRAM» | `host.rs` +14 (`dedicated_video_memory`), `resource.rs` +71/−6 (проактивный дефолт + тест) | **у нас нет подсистемы mip-drop вообще**: `TEXTURE_MIP_DROP`/`MAX_MIP_DROP`/`OOM_MIP_PRESSURE` в нашем коде отсутствуют (grep: 0 совпадений). Их патч — смена **дефолта** поверх готовой подсистемы (аллокация текстуры меньше запрошенной + pruning `CopyTextureRegion`). Перенос = написать эту подсистему (500+ строк, риск артефактов) — не «маленькая правка» |
| [techselect 360f3825](https://github.com/techselect/VIctory/commit/360f38256dea4e80f5b3afd84227b61462b9b6a9) «титры + утечка + GPU hangs» | 14 файлов, ~1 328 строк диффа: `lower.rs` (много хунков, +129/−53), `emit/mod.rs`, `host.rs` +142, `draw.rs` +44, `resource.rs` +86, `shadow.rs` +28, `xmem.rs`, `indirect.rs`, `list.rs` [и др.] | меняет **эмиттер шейдеров** (`victory-shader`), то есть 25 635 прогретых переводов перестали бы совпадать по содержимому и весь шейдерный кэш надо греть заново (7 мин); в одном коммите смешаны падения GPU, утечка памяти, mesh-экспорты и текст титров; хуже того, правки — против апстримных файлов, а у нас своя линия (`draw.rs` с `heap_textures`, `shadow.rs` с `relay`/`wide_back`) |

Итог: **ничего не портировано**, поэтому `cargo check --workspace`/`cargo test --workspace` на этом
шаге не запускались (код не менялся; последние зелёные прогоны — §9). К идее mip-drop можно
вернуться только как к отдельной задаче «подсистема mip-drop», не как к порту 85 строк.

### 11.2. Релизная сборка

```powershell
cargo build --release -p victory
```
`Finished release profile [optimized + debuginfo] in 55,56 s` (инкрементально, без `cargo clean`).
Артефакт: `target\release\victory.exe`, 8 481 280 байт, 04:35:27. `victory-shader` не менялся →
прогретый релизный кэш (25 635 переводов, §5) остаётся валидным, повторный прогрев не нужен.

### 11.3. Прогон 3 (релиз, 1080p, флаги разгрузки VRAM)

Скрипт `C:\GTA6_Port\run-release.ps1`; наблюдатель `watch-release.ps1` (титул окна с fps, `nvidia-smi`
раз в 2 с, скриншот окна раз в 30 с); клавиши — `sendkeys2.ps1`.
Флаги: флаги скорости из §7 (1080p, ASYNC_SUBMIT, QUEUE2, INDIRECT_WINDOW, TEXTURE_NO_WAIT,
MERGE_GAP=64, MERGE_RUNS=4096, STAGING_POOL=0, COPY_PLAIN, JOIN_PARTS, BACK_OWN_VIEW,
INDIRECT_GATHER, CARD_REHASH_RUNS, HOST_WAVE=32, VIEW_LIMIT=999999) плюс разгрузка VRAM:
`VICTORY_TEXTURE_BUDGET_MB=2500`, `VICTORY_REUPLOAD_WHEN_FULL=0`, `VICTORY_TEXTURE_STAGE_MB=64`,
`VICTORY_TEXTURE_STAGE_MS=80`, `VICTORY_TEXTURE_CATCHUP=0`; `VICTORY_LOG=info`.

| Что | Значение |
|---|---|
| Запуск/останов | 04:35:41 → 04:46:36 (**655 с**), PID 17000, закрыт `CloseMainWindow` (процесс не вышел сам, добит) |
| 3D-мир | **с 04:37:47 (t≈126 с)**, 86 кадровых строк с `draws ≥ 200`, из них 70 «устоявшихся» (после первых 120 с) |
| EULA | **не встретилась** — она была принята в прогоне 2 и сохранилась в профиле; игра сама ушла с титульного экрана в интро (attract) на ~120-й секунде |
| Клавиши | первый скрипт нажатий **не работал**: переменная `$pid` в PowerShell — автоматическая, запись молча падала, и скрипт слал клавиши сам себе; плюс Windows не отдаёт фокус. Починил в `sendkeys2.ps1`: Alt-разблокировка + minimize/restore + `AttachThreadInput`, проверка фокуса. После фикса `Enter` ушёл в игру (фокус = 17000), но геймплей уже начался сам |
| Логи/кадры | `C:\GTA6_Port\run-logs3\` (`err.log` 4,46 МБ, `out.log`, `captions.log`, `vram.log`, `keys.log`, 20+ `shot-*.png`) |
| Ошибки | **0 строк ERROR**; 269 WARN: 219 × «a texture picked by index changed after its upload — uploaded again», 6 × «host pipeline refused», 1 × «the card has no room for more», остальное — мелочь |
| Артефакты | чёрный кадр в интро на t≈504 с (`shot-504.png`: 0,5 fps, 495 draw, виден только субтитр) — тот же артефакт, что в §8; текст титров на остальных снимках читается нормально, наложения, как в кадре 4233, не поймали |

### 11.4. Замеры

**FPS в мире** (кадровые строки, `draws ≥ 200`): медиана **1,5 fps**, среднее 1,61, min 0,1, max 5,0;
в «устоявшейся» части (70 строк): медиана **1,7 fps** (кадр 596 мс). По 100-секундным корзинам мира:
1,2 → 1,5 → 1,6 → 1,2 → **2,1** → 1,8 fps (лучшая — 400–500 с).

**Из чего кадр** (медианы; в скобках — прогон 2, диагностическая сборка, §8):

| Компонент | Прогон 3 (релиз, 70 строк) | Прогон 2 (diag, 48 строк) |
|---|---|---|
| кадр | 596 мс | 506 мс |
| preparing | 116 мс | 98 мс |
| — copies | 67 мс / 82 МБ | 41,8 мс / 63 МБ |
| — textures | 14,8 мс / 17 МБ | 14,3 мс / 20 МБ |
| GPU | 0 мс | 0 мс |
| copying back | 70,9 мс / 18,6 МБ | 32,2 мс / 7,6 МБ |
| проход по куче (heap buffers) | **134 мс** | 182 мс |
| draws / брошено / пайплайнов делается | 990 / 359 / 618 | 700–1000 / 240–937 / ~950 |

**VRAM** (`nvidia-smi`, карта 8 192 МБ; фон рабочего стола до запуска — 1 335 МБ):
среднее 6 523 МБ, **пик 7 923 МБ = 96,7 %**; по корзинам мира медиана 6 943 / 7 510 / 7 043 / 7 546 /
7 365 МБ; «только игра» ≈ 5,2 ГБ в среднем и **~6,6 ГБ в пике**. По D3D12 на загрузке мира:
бюджет процесса 6 339 МБ, использовано 4 803 МБ.
**«passes that made none for lack of room»: 0** (в прогоне 2 — **1 520**) — потолок
`VICTORY_TEXTURE_BUDGET_MB=2500` сработал: отказов по месту больше нет. Но очередь текстур по
индексу **не рассосалась**: 60 697 загружено за прогон (21,8 ГБ), **16 782 всё ещё ждут**.

**Пайплайны/шейдеры**: за прогон создано **3 227 PSO** («first bind pipeline»), 2 521 перевод
шейдера в рантайме, 30 инкрементальных сохранений библиотеки пайплайнов. Поток создания пайплайнов
— главный «не наш» потребитель кадра: медиана 618 пайплайнов «делается» и 359 брошенных draw/кадр.

### 11.5. Выводы

1. **Флаги разгрузки сработали частично:** проход по куче подешевел на **~48 мс** (182 → 134 мс),
   отказы «нет места» ушли (1 520 → 0), но **пик VRAM не снизился: 96,7 %** — потолок держат не
   текстуры по индексу (их теперь кэп 2,5 ГБ), а остальное: render target'ы, host-копии под запись
   карты, staging, пайплайны. Очередь 16 782 текстур означает, что 64 МБ/80 мс staging слишком
   медленно для этого профиля (медленнее, чем просит сцена).
2. **FPS не вырос:** медиана 1,7 fps в мире против ~1,86 среднего в прогоне 2. Причины роста кадра —
   больше отданных назад байт (18,6 МБ/кадр против 7,6), больше draw (990), и всё тот же шторм
   создания пайплайнов (3 227 PSO; 600+ «делается» каждый кадр, 359 draw брошено).
3. **Релиз против diag выигрыша не дал** — значит instrumentation диагностической сборки на этом
   участке не была главным тормозом (копии/копирование назад съедают столько же или больше).
4. Чёрные кадры в интро **воспроизводятся и в релизе** (t≈504 с) — это не артефакт diag-сборки.
5. EULA больше не мешает: она принята и сохранена в профиле; при следующих прогонах меню проходит
   само (attract), а `sendkeys2.ps1` остаётся на случай нового экрана/сброса профиля.

### 11.6. Следующий шаг (нужно решение)

1. **Повторить прогон релиза подряд** (10 минут): библиотека пайплайнов уже записана (79 МБ),
   шейдеры — тоже; второй запуск должен иметь меньше «первых bind» и выше fps. Это отделит
   «холодный старт» от реального потолка.
2. **Вернуть staging к 128–192 МБ / 160–250 мс** (или `VICTORY_TEXTURE_CATCHUP=1`), чтобы
   рассосать 16 782 текстуры; связать с fps и числом «passed over unmapped» (сейчас 8 718).
3. **Prefetch-фикс copy-back** (real69xaros, +37 строк): наш `copying back` — 71 мс/кадр и
   профиль page fault'ов тот же, что у них (лечили 145 из 263 мс). Это единственная дешёвая правка,
   прямо бьющая в 71 мс.
4. **Чёрные кадры** — снять `VICTORY_CAPTURE_FRAMES` вокруг t≈504 с диагностической сборкой и
   смотреть `frame-<n>-ops.txt`; по форкам есть `bodnjenie14` (Bink) и `NitricAcidDev` (intro/overlay),
   но это ручной перенос апстрим-кода.
5. Титры: на снимках текст читается, наложение не поймано — пока отдельно не чиним.

### 11.7. Файлы этого шага
- `C:\GTA6_Port\upstream\esat-mipdrop.diff`, `techselect-credits.diff` — разобранные коммиты;
- `C:\GTA6_Port\run-release.ps1` — запуск релиза с флагами VRAM; `watch-release.ps1` — титул/fps,
  `nvidia-smi`, скриншоты; `sendkeys2.ps1` — реальные нажатия с надёжным фокусом (рабочий вариант);
  `press-release.ps1` — неудачный первый вариант (оставлен для истории);
- `C:\GTA6_Port\run-logs3\` — `err.log` (4,46 МБ), `out.log`, `captions.log`, `vram.log`, `keys.log`,
  `shot-*.png` (20+), `pid.txt`, `started.txt`.

## §12. Прогон 4: релиз на прогретой библиотеке пайплайнов (2026-11-26, код не менялся)

Задача 1 ночной сессии: повторить прогон релизной сборки подряд (библиотека пайплайнов уже на диске),
ослабить зажим стриминга текстур до средних значений и замерить FPS/VRAM/очередь, проверить чёрный кадр.

### 12.1. Флаги и ход прогона

Скрипты: `C:\GTA6_Port\run-release2.ps1` (запуск), `watch-release2.ps1` (титул/fps, `nvidia-smi` раз в 2 с,
скриншот окна раз в 30 с), `sendkeys2.ps1` (реальные нажатия), логи — `C:\GTA6_Port\run-logs4\`.
Флаги — те же, что в §11 (1080p и набор скорости), `VICTORY_TEXTURE_BUDGET_MB=2500`,
`VICTORY_REUPLOAD_WHEN_FULL=0`, а стриминг текстур ослаблен до среднего: **`VICTORY_TEXTURE_STAGE_MB=128`,
`VICTORY_TEXTURE_STAGE_MS=160`, `VICTORY_TEXTURE_CATCHUP=1`**. Сборка — тот же релиз из §11 (`victory-shader`
не менялся, повторного прогрева не было; библиотека пайплайнов — от прогона 3).

| Что | Значение |
|---|---|
| Запуск/останов | 05:07:24 → 05:18:04 (**640 с**), PID 760, остановлен `Stop-Process` (сам не вышел) |
| Клавиши | `sendkeys2.ps1` сработал сразу: `Enter` в t≈25 и t≈55 с доставлены при `foreground pid=760` |
| 3D-мир | **с t≈87 с** (в прогоне 3 — со 126 с: там нажатия не дошли, игра шла через attract) |
| Куда дошла | **дальше, чем в прогоне 3**: не титры-заставка на фоне города, а сюжетные катсцены с персонажами (тёмные интерьеры/улица, 1920x1080; см. `shot-511.png`, `shot-541.png`) |
| Логи/кадры | `err.log` 20291 строк, `captions.log` 620, `vram.log` 301, `keys.log`, 20 `shot-*.png` |
| Ошибки | **0 ERROR**; 286 WARN: 236 × «a texture picked by index changed after its upload — uploaded again», 29 × «a pipeline creation method that is not implemented: the title goes without this pipeline stage», 7 × «host pipeline (mesh/compute) refused», 1 × «the card has no room for more», остальное — единичные |

### 12.2. FPS

**Мир** (96 кадровых строк с `draws ≥ 200`, t=87…618 с): медиана **2,0 fps**, среднее 2,76, min 0,2, **пик 10,9**;
в «устоявшейся» части (74 строки, t≥207 с) медиана **1,8 fps** (554,5 мс/кадр), среднее 2,58.
Пик в меню/загрузке — 30,2 fps (10 draw/frame). По 100-секундным корзинам мира:
2,5 → 2,1 → 2,3 → 1,8 → 1,4 → 1,9 fps (лучшая — первые 100 с, худшая — 487–587 с, там переходы сцен).

Сравнение с прогоном 3 (§11.4), та же сборка и те же флаги, кроме стриминга:

| | Прогон 3 (64/80, catchup=0) | Прогон 4 (128/160, catchup=1) |
|---|---|---|
| кадров мира / медиана fps | 86 / **1,5** | 96 / **2,0** |
| «устоявшаяся» медиана | 1,7 fps (596 мс) | **1,8 fps** (554 мс) |
| пик fps в мире | 5,0 | **10,9** |
| медиана draw/frame | 795 | 2058 (контент тяжелее: катсцены, до 7568 draw) |
| медиана «пайплайнов делается» | **702** | **0** |
| медиана «брошено без пайплайна» | 472 | 290 |
| copies / textures (медиана) | 70,2 мс / 86 МБ, 15,2 мс / 18 МБ | 66,6 мс / 125 МБ, 20,1 мс / 34 МБ |
| copying back (медиана) | 74,8 мс / 19 МБ | 68 мс / 45 МБ |
| heap-проход (медиана) | 140,2 мс | 147,5 мс |

То есть **медиана и пик выросли, хотя контент стал тяжелее** (вдвое больше draw на кадр, втрое больше
отданных назад байт): главный выигрыш — тёплая библиотека пайплайнов: шторма создания пайплайнов в мире
больше нет (`making>50` — 0 кадров из 96, у прогона 3 — медиана 702). «Первый bind» за прогон — 3745 PSO,
но это уже загрузка/создание заранее (`host pipelines all made made=304 loaded=2262 started_early=2204`),
а не лавина посреди кадра.

### 12.3. Очередь текстур и VRAM

Очередь текстур на конец прогона (`textures picked by index, the queue` в 00:18:00 UTC):
**15622 слота** в очереди, из них **2392 «просроченных»** (самый старый ждёт **67 кадров**), **5518 ждут
своей памяти**, 2886 загружены частично; по счётчикам кадра: за прогон загружено **124 255 текстур
(52 993 МБ = 53 ГБ)**, 15 622 ждут, 35 348 «пройдено мимо несмапленного».
В прогоне 3 было: 16 782 слота, **12 912 просроченных (самый старый — 1587 кадров!)**, 1916 ждут памяти,
загружено 60 697 (21,8 ГБ), 16 782 ждут, 8718 пройдено мимо.

**Вывод по стримингу:** средние значения staging + `CATCHUP=1` *рассосали* просроченный хвост очереди
(12 912 → 2 392, возраст старейшего 1587 → 67 кадров) и удвоили пропускную способность (21,8 → 53 ГБ),
но «ждущих памяти» и «пройдено мимо несмапленного» стало больше (1916 → 5518 и 8718 → 35 348): теперь
лимит — не бюджет staging, а наличие/смапленность памяти самих текстур. Отказов «нет места» по-прежнему
**0** (в прогоне 2 было 1520), «the card has no room for more» — 1 раз, ещё до мира (t≈81 с).

**VRAM:** 301 замер `nvidia-smi`: медиана **7560 МБ**, среднее 6902, **пик 7890 МБ = 96,3 %** из 8192
(у прогона 3 пик 7923 = 96,7 %); утилизация GPU — медиана 20 %, максимум 100 %. То есть ослабление
стриминга VRAM не разгрузило (даже чуть подняло медиану: больше текстур на карте) — потолок держат
render target'ы, host-копии под запись карты, staging и пайплайны, а не бюджет текстур по индексу.

### 12.4. Чёрный кадр в интро

В прогоне 3 чёрный кадр был на t≈504 с (`shot-504.png`, средняя яркость содержимого окна **0,0** —
чистый чёрный плюс только наша FPS-плашка); рядом, на t≈499 и 506 с, в логе кадры 3024,9 и 7609,7 мс
с **dropped 1049/1094 (549/530 пайплайнов делается)** — переход к новой сцене.

В прогоне 4 из 20 скриншотов (раз в 30 с) **ни один не чёрный**: минимум средней яркости — 4,1
(`shot-210.png`, тёмная катсцена в интерьере с видимыми персонажами), у прогона 3 чёрных снимков было
три (121 и 151 с — экраны загрузки мира, 504 с — интро). Тот же тяжёлый переход в прогоне 4 случился
на t≈519–537 с: кадр 6378 мс, dropped 3190, но 3 пайплайна делается, и на снимке t=541 с уже видна
сцена (герои), а не чёрный экран. **Но**: маршрут прогона 4 отличался (нажатия дошли → игра пошла по
сюжету, а не через attract с титрами-заставкой), поэтому «чёрный кадр именно на t≈504» в этом прогоне
не воспроизводился. Честный итог: **в этом прогоне чёрных кадров не поймано, шторм создания пайплайнов
в мире исчез, но один многсекундный переход (6,4 с) остался** — его причина разобрана в §14.

### 12.5. Выводы

1. Тёплая библиотека пайплайнов — **главный выигрыш**: медиана мира 1,5 → 2,0 fps, пик 5,0 → 10,9 fps,
   при вдвое более тяжёлом контенте; «пайплайнов делается» медиана 702 → 0.
2. Средний стриминг (128 МБ / 160 мс / catchup=1) **чинит хвост очереди** (просрочено 12 912 → 2 392),
   не снижая FPS; цена — больше текстур на карте (медиана VRAM 7560 МБ) и больше «ждущих памяти»
   (лимит — смапленность памяти текстур, а не staging).
3. VRAM по-прежнему 96 %: ручками стриминга её не разгрузить, нужны меры по render target'ам/карте.
4. Игра впервые дошла до **сюжетных катсцен с персонажами** — это уже не «титры на фоне города».
5. Чёрных кадров в прогоне 4 не поймано, но маршрут отличался; причина класса «чёрный экран на
   переходе» — §14.

### 12.6. Файлы этого шага
- `C:\GTA6_Port\run-release2.ps1`, `watch-release2.ps1` — запуск/наблюдение прогона 4;
- `C:\GTA6_Port\analyze4.ps1` — разбор кадровых строк и `nvidia-smi`; `shots4.ps1` — яркость снимков;
- `C:\GTA6_Port\run-logs4\` — `err.log`, `out.log`, `captions.log`, `vram.log`, `keys.log`, 20 `shot-*.png`,
  `pid.txt`, `started.txt`.

## §13. Prefetch-фикс copy-back (2026-11-26, код изменён — единственная правка ночи)

### 13.1. Источник и идея

Коммит [`real69xaros/victory-laptop 8e08d08f`](https://github.com/real69xaros/victory-laptop/commit/8e08d08f2348765120497e1f5410c6e3f12dd00e)
(дифф скачан в `C:\GTA6_Port\upstream\real69-prefetch.diff`, 2408 Б, `shadow.rs` +37 строк; файл
`real69-shadow.rs` — для сверки): в их замере 49 745 page fault'ов на кадр стоили **145 из 263 мс** кадра;
перед сравнением кусков копии вызывается `PrefetchVirtualMemory` — ядро подгружает диапазон одним вызовом,
а не по странице за раз. Выключатель — `VICTORY_PREFETCH=0`.

### 13.2. Как перенесено (+55 строк в `crates/victory-gpu/src/shadow.rs`)

Голый апстримный хунок (`Self::prefetch(...)` внутри `copy_back_with`, только при `found.is_none()`) у нас
был бы **мёртвым кодом**: при `VICTORY_RELAY=2` (по умолчанию) copy-back считает поток relay —
`relay_hand_back` (`host.rs:3387`) → `shadow::find_all` → `find_pieces`, и `found` приходит уже готовым.
Поэтому порт сделан в трёх местах:

1. **Свободная функция `prefetch(address, len)`** (`shadow.rs`, +32 строки) — та же Win32-функция
   `PrefetchVirtualMemory` через `#[link(name = "kernel32")]`, флаг `VICTORY_PREFETCH` в `OnceLock`,
   guard `len == 0`; оформлена как подсказка: ошибка вызова ничего не меняет.
2. **`find_pieces`** — перед запуском параллельного сравнения по страницам подгружаются разом страницы
   **байтов копии и снапшота** каждого диапазона (`data + from`, `before + from`) — это ровно те страницы,
   которые читает сравнение в нашем дефолтном relay-пути. Это и есть рабочий вызов на каждом кадре.
3. **`copy_back_with`** — апстримный вызов как есть (страницы title-памяти) для пути без relay/wide,
   где сравнение идёт "по месту" (`found = None`).

Итого: `crates/victory-gpu/src/shadow.rs` +55 строк, других файлов нет. `victory-shader`/`emit/mod.rs`
не тронуты → ключ и содержимое прогретого кэша шейдеров (25 635 переводов) не меняются, перегрев не нужен.

### 13.3. Обязательные проверки (AGENTS.md)

- `cargo check --workspace` — **exit 0** (11,98 с; единственное предупреждение — старый `placed_filler_twins`).
- `cargo test --workspace` — полный прогон **зелёный, exit 0**: victory-core 11, victory-formats 25 (+4 ignored),
  victory-gpu **184** (+6 ignored), victory-hle 100, victory-loader 27+2+1, victory-pe 7, victory-pipeline 4,
  victory-shader 216 (+3 ignored), xtask 4, doc-tests 0.
- **Про флаки (важно, чтобы не принять чужое за поломку):** первый и второй прогоны упали на
  *пре-существующих* нестабильных тестах GPU/памяти — `inplace::tests::warp_a_title_buffer_in_place_...`
  (assert на `VirtualFree(MEM_DECOMMIT)`) и `STATUS_ACCESS_VIOLATION` в guard-тестах `watch`; правка к ним
  не относится. Проверено базой: правка отложена (`git stash`), `cargo test -p victory-gpu --lib` без неё
  упал **2 из 2** прогонов на других `warp_*`-тестах. После возврата правки полный прогон — зелёный.
  Стоп-кран «3 падения подряд по вине правки» не сработал: падения не из-за правки.
- `cargo build --release -p victory` — **exit 0**, `Finished release profile ... in 32,60 s`;
  `target\release\victory.exe` — 8 488 960 байт, 05:29:01 (собран заново, уже с фиксом).

### 13.4. Замечания и ожидаемый эффект

- Семантика копирования не меняется: `prefetch` только подгружает страницы, ничего не пишет и не
  освобождает; гонок не добавляет. В `find_pieces` вызов стоит до сравнения — страницы копии/снапшота
  уже гарантированно живы (на них же ссылаются слайсы сравнения).
- Ожидаемый эффект — сокращение page-fault'ов в «copying back» (в прогоне 4 медиана 68 мс, в 3-м — 71 мс;
  в отдельных кадрах 119–163 мс); проверять следующим прогоном, A/B через `VICTORY_PREFETCH=0`.
- В этом шаге полётного прогона не делал (задача 2 просила только порт + check/test/релиз).

### 13.5. Файлы
- `C:\GTA6_Port\upstream\real69-prefetch.diff`, `real69-shadow.rs` — исходный коммит и его файл;
- `crates\victory-gpu\src\shadow.rs` — правка (+55 строк, 3 места выше);
- `target\release\victory.exe` — пересобранная релизная сборка с фиксом.

## §14. Чёрный кадр в интро: разбор причины (2026-11-26)

Проверял без переписывания рендера: по снимкам, логам прогонов 3/4 и коду. Ниже — что происходит и почему.

### 14.1. Что именно на снимках

- `run-logs3\shot-504.png` (t≈504 с) — **чистый чёрный экран**, видна только наша плашка FPS
  (0,5 fps, 495 draw). Средняя яркость содержимого окна — **0,0**.
- `run-logs3\check2.png` (t≈234 с, снят вручную) — **чёрный фон + читаемый текст титров**
  («Written By: Rupert Humphries …»), 1,2 fps, 656 draw. То есть текст на чёрном **рисуется нормально**.
- Соседние снимки 473 и 534 — тот же город с титрами (фон городской), т.е. интро — это «карточки»
  титров: текст на фоне города, между группами экран/фон уходит в чёрное.
- Скриншоты загрузки мира (t≈121 и 151) тоже чёрные — это штатные экраны загрузки самой игры.

### 14.2. Что говорит лог в момент чёрного кадра (прогон 3)

t≈499–506 с (UTC 23:44:00–23:44:08): кадры **3024,9 мс** и **7609,7 мс**, в них
`dropped 1049 (549 being made)` и `dropped 1094 (530 being made)` — то есть половина draw'ов кадра
(2209 и 2156) **не нарисована**, потому что их host-пайплайны ещё делаются; `indirect mesh dispatches`
просит максимум (65 535 групп, 127 757 групп за кадр) — это новая сцена, которую игра готовит в этот момент.
Через 6 с (t≈510–514) снова 977–820 draw и нормальные кадры. Отказов «нет места» в эти секунды нет,
ошибок ERROR нет, текстуры в кадре грузятся (104,9 мс / 105 МБ), GPU 0 мс.

В прогоне 4 чёрных снимков нет (минимум яркости 4,1 — тёмная игровая катсцена), но подобный переход
есть на t≈519–537 с: кадр **6378 мс**, `dropped 3190`, при этом **3** пайплайна делается (в прогоне 3 было
530–549) — тёплая библиотека сняла лавину, но 3 дорогих пайплайна (mesh/compute) всё ещё дают 6,4 с.

### 14.3. Причина — точная, в коде

1. **Чёрное — это кадр самой игры (затухание/карточка титров), а не наша очистка.** Доказательство:
   `check2.png` — чёрный фон с корректно нарисованным текстом титров; наша плашка FPS рисуется поверх;
   никакого «бага clear» нет. Город/фон на время перехода игра гасит сама — так свёрстаны титры.
2. **Почему это висит секундами (и почему на 504-м нет даже текста).** У нас host-пайплайны делаются
   асинхронно: первый bind ставит пайплайн в очередь и ждёт его **в бюджете 0,5 с на кадр**; кто не успел —
   draw **бросается** (`crates/victory-gpu/src/pipeline/make.rs:10-16` — «a draw or dispatch whose pipeline
   is not ready is dropped… the default budget is half a second per title frame»; сам drop —
   `draw.rs:1016` `host_pipeline(...) == None → return false`, счётчик `DROPPED` — `draw.rs:2944-2948`).
   В момент смены сцены игра просит сотни новых пайплайнов; каждый — перевод шейдера + компиляция драйвером
   (по комментарию в `make.rs:3-8` — от миллисекунд до **3,4 с** на драйверную компиляцию, «the title's first
   world frame wants over 1 200 new pipelines»). Итог: кадр длится 3–7,6 с, видимая геометрия этого кадра
   в основном в `dropped`, а на экране остаётся чёрный переход. Один кадр = секунды → затухание в чёрное,
   которое в нормальном темпе заняло бы доли секунды, растягивается на 5–10 с.
3. **Нехватка VRAM тут ни при чём:** в эти секунды нет ни одного сообщения «нет места»/«made none for lack
   of room» (в прогоне 3 их 0), текстуры грузятся, ERROR — 0. VRAM в прогоне 3 была тяжёлой (пик 96,7 %),
   но к чёрному кадру это не относится: кадр не ждёт память, он ждёт пайплайны.
4. Это **не «PSO потерян навсегда»**: следующий кадр пайплайны уже сделаны, и сцена рисуется (t≈510+).
   Потерян только текущий кадр перехода — отсюда чёрный/пустой кадр в интро и «выпавшие» титры.

### 14.4. Почему в прогоне 4 этого (почти) нет

Тёплая библиотека пайплайнов: медиана «пайплайнов делается» 702 → **0**, «брошено» 472 → 290
(§12.2). Чёрных снимков в прогоне 4 нет; переход на t≈537 дал 6,4 с, но с 3 пайплайнами вместо 530–549,
и на экране держался предыдущий (яркий) кадр. Вывод: лечится это **не** ручками VRAM и не переписыванием
очистки, а прогревом/предзагрузкой пайплайнов (что и дал второй запуск), и частично — увеличением
библиотеки раннего прогрева (`note_used`/`wanted_early` в `pipeline/make.rs:120-147`).

### 14.5. Что можно сделать дальше (не делал, на утреннее решение)
1. Прогнать A/B `VICTORY_PREFETCH=0` после §13 — на «copying back»/fps.
2. Догреть пайплайны: 2–3 запуска подряд с прохождением одних и тех же катсцен (библиотека
   `wanted_early`/`started_early` уже пишет, что нужно греть) — чёрные переходы должны сократиться.
3. Если чёрные на переходах останутся — снять `VICTORY_CAPTURE_FRAMES` diag-сборкой вокруг перехода и
   посмотреть `frame-<n>-ops.txt`: видно, какие именно draw в `dropped` (нужен повтор маршрута).

### 14.6. Файлы
- `C:\GTA6_Port\run-logs3\shot-504.png`, `check2.png`, `shot-473.png`, `shot-534.png`;
- `C:\GTA6_Port\run-logs4\shot-*.png`; `err.log` прогонов 3 и 4 (кадровые строки, `dropped … being made`);
- код: `crates\victory-gpu\src\pipeline\make.rs` (бюджет ожидания и drop), `crates\victory-gpu\src\draw.rs`
  (`prepare_for` → `return false`, счётчик `DROPPED`).

## §15. Продолжение: A/B prefetch-фикса, три коротких прогона, итог ночи (2026-11-26)

Продолжение по §14.5 п.1: проверить prefetch из §13 в полёте. Сделаны три прогона по 350 с на одном и том же
релизе (фикс в сборке), с флагами прогона 4 (1080p, `BUDGET_MB=2500`, `REUPLOAD_WHEN_FULL=0`, staging
128/160/catchup=1), тёплая библиотека пайплайнов, клавиши `Enter` на t≈20 и 50 с. Логи — `run-logs5/6/7`.

| Прогон | prefetch | лог | мир с | кадров мира | draws (мед.) | fps мед. / уст. | кадр мс | copy-back | copies | textures | heap | VRAM мед./пик |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| A | **вкл** | `run-logs5` | 121 с | 41 | 3671 | 4,8 / 5,5 | 207,4 | 77,9 мс / 64,6 МБ (**1,22 мс/МБ**) | 57,1 мс / 109 МБ | 10 мс / 20,4 МБ | 66,7 мс | 7201 / 7636 МБ |
| B | **выкл** | `run-logs6` | 144 с | 39 | 4199 | 5,9 / 6,4 | 168,5 | 64,4 мс / 66,5 МБ (**0,99 мс/МБ**) | 44,5 мс / 103 МБ | 7,6 мс / 17,2 МБ | 49,0 мс | 7466 / 7900 МБ |
| A2 | **вкл** | `run-logs7` | 139 с | 37 | 4164 | 5,6 / 6,25 | 179,3 | 68,4 мс / 64,9 МБ (**1,11 мс/МБ**) | 47,2 мс / 102 МБ | 9,7 мс / 19 МБ | 64,2 мс | 7289 / 7919 МБ |

### 15.1. Что показал замер

- Прогон A **неравноценен** и как свидетель отбрасывается: у него окно с t≈120 с схлопнулось и снимки
  писались 160×28 (артефакт наблюдателя, не игры), контент легче (3671 draw против 4199) — вероятно,
  прогон был чем-то потревожен.
- Честная пара — **B (выкл) и A2 (вкл)** при почти одинаковом контенте (4199 и 4164 draw, мир с 144 и 139 с).
  С prefetch: **fps 5,6 против 5,9**, copy-back 68,4 против 64,4 мс, копии 47,2 против 44,5 мс,
  текстуры 9,7 против 7,6 мс, heap-проход 64,2 против 49,0 мс — то есть включённый prefetch **не быстрее,
  а чуть медленнее** по всем метрикам.
- Но разброс между **одинаковыми** сборками (A и A2, обе с prefetch) — 4,8 против 5,6 fps (~17 %), то же
  и по прочим метрикам. Поэтому «prefetch замедляет» **не доказано**, а «prefetch ускоряет» — не подтверждено.
- Правдоподобное объяснение: наш перенос prefetch-ит **страницы копии и снапшота**, которые в relay-пути и
  так горячие (их только что писал GPU/refresh), а апстримный выигрыш был на **title-страницах** перед
  сравнением в их ветке без relay. То есть «правильный» перенос — prefetch title-диапазона в
  `relay_hand_back` перед `find_all`, а не тот, что сделан.

### 15.2. Решение

- Код §13 **оставлен** (свободная функция + три вызова), но переведён в **opt-in**: `VICTORY_PREFETCH=1`
  включает, по умолчанию **выключен** (комментарий в `shadow.rs` обновлён). Так дефолтная сборка не получает
  неподтверждённого замедления, а эксперимент можно повторить одной переменной.
- Сборка обновлена: `cargo build --release -p victory` — 26,15 с, `target\release\victory.exe` 8 488 960 Б,
  06:07:39.
- Проверки после смены дефолта: `cargo check --workspace` — exit 0; `cargo test --workspace` — **зелёный
  (exit 0, 3-я попытка)**: gpu 184, hle 100, shader 216, core 11, formats 25, loader 27+2+1, pe 7,
  pipeline 4, xtask 4. Попытки 1–2 упали на **тех же** пре-существующих флаки warp-тестах
  (`inplace::..._in_place_...`, `indirect::warp_budget_...`, один раз `STATUS_ACCESS_VIOLATION`) — это те
  самые тесты, что падают и **без** правки (проверено `git stash`-базой в §13.3), к prefetch отношения не имеют.

### 15.3. Чёрные кадры и общее поведение в этих прогонах

- Снимки B и A2 — рабочие (1920×1080), **ни одного чёрного**: минимум яркости 33,4 (тёмная сцена старта),
  дальше 51–226. У прогона A снимки с t≈120 битые (окно 160×28), для проверки не годятся.
- Тёплая библиотека держится: `making` = **0** во всех трёх прогонах, `dropped` 382–423, fps 4,8–6,4 —
  то есть после прогрева игра устойчиво идёт в 2–3 раза быстрее, чем в первых прогонах (§12: 1,8–2,0).
- Чёрных переходов в этих прогонах не было — согласуется с §14: причина была в шторме создания PSO,
  а он после прогрева отсутствует.

### 15.4. Итог ночи и состояние репозитория

- Ветка `perf-and-upstream-fixes`, `master` не тронут. Изменены 5 файлов: §9 (`descriptor/host.rs`,
  `draw.rs`, `pipeline/library.rs`, `resource.rs`) + §13/§15 (`shadow.rs`, prefetch opt-in).
  `AGENTS.md` и `HANDOFF.md` — вне git (untracked), лежат на диске.
- Релизная сборка — с фиксом §9 и opt-in prefetch; 8 488 960 Б (06:07:39).
- Все прогоны остановлены, `victory.exe` в памяти нет. ПК выключается по просьбе пользователя
  (ночная сессия завершена).
- Что осталось на будущее (не делал): 1) prefetch title-страниц в `relay_hand_back` + A/B с `VICTORY_PREFETCH=1`;
  2) догрев пайплайнов ещё 1–2 запусками (библиотека уже растёт); 3) разгрузка VRAM (render target'ы /
  host-копии / пайплайны) — отдельная задача; 4) diag-capture переходов, если чёрные вернутся.

### 15.5. Файлы
- `C:\GTA6_Port\run-ab.ps1` — запуск A/B (`-Prefetch 1/0`), `analyze-ab.ps1` — таблица A/B/A2,
  `shots4.ps1` — яркость снимков;
- `C:\GTA6_Port\run-logs5\`, `run-logs6\`, `run-logs7\` — `err.log`, `captions.log`, `vram.log`,
  `keys.log`, `shot-*.png`, `pid.txt`, `started.txt`, `prefetch.txt`;
- `C:\GTA6_Port\upstream\real69-prefetch.diff`, `real69-shadow.rs` — исходный коммит.

## §16. Дневная сессия: бэкап-репозиторий и разведка по пересвету/теням/FPS (2026-11-26, код не менялся)

Живой прогон пользователя дошёл до выхода из тюрьмы. Три симптома: (1) сильный пересвет — окна, лампы,
спина персонажа выжигаются в белый, остальная комната тёмная; (2) нет непрямого света и нормальных теней;
(3) в живом геймплее 1–3 FPS и мыльные/недогруженные текстуры. Ниже — что найдено и что можно взять.

### 16.1. ЭТАП 1 — приватный бэкап (сделано)

```powershell
gh repo create 4EKUSHKAq/victory-fork-3060ti --private --description "Private backup ..."
gh auth setup-git
git remote add myfork https://github.com/4EKUSHKAq/victory-fork-3060ti.git
git push -u myfork perf-and-upstream-fixes master
```

- Репозиторий: https://github.com/4EKUSHKAq/victory-fork-3060ti — **private**, владелец `4EKUSHKAq`,
  default branch `master` (первым запушен master).
- Запушено: `perf-and-upstream-fixes` (на момент пуша — `8f12565`, далее коммит с §16) и `master`
  (`d31f841`). `git remote -v`: только `myfork` (исходных remote у локального репозитория нет).
- `HANDOFF.md` и `AGENTS.md` были untracked — закоммичены в `perf-and-upstream-fixes` вместе с этим
  отчётом, чтобы §16 и правила лежали в бэкапе. Код в этом шаге не менялся.
- Замечание: сразу после создания GitHub отдавал репозиторий как **private**, но при проверке через
  10–15 минут API показал `visibility: public` (причина не установлена; возможно, гонка при создании).
  Исправлено: `gh repo edit 4EKUSHKAq/victory-fork-3060ti --visibility private --accept-visibility-change-consequences`;
  повторная проверка — `{"isPrivate":true,"visibility":"PRIVATE"}`. Секретов/игровых файлов в репозитории
  нет (только код VIctory GPL-3.0 + `AGENTS.md`/`HANDOFF.md` с локальными путями).

### 16.2. Наш срез и что в нём есть

- Наш коммит-снимок `d31f841` создан 2026-11-26 02:37 +05:00 (**2026-11-25 21:37 UTC**).
- В нашем дереве **235** переключателей `VICTORY_*`. Среди них по теме освещения только BVH:
  `VICTORY_BVH_BUILDER`, `VICTORY_BVH_QUARANTINE`, `VICTORY_BVH_HOLD`.
- **Переключателей HDR/тонемаппинга/экспозиции/цвета нет вообще**: нет `VICTORY_HDR*`, `*TONE*`,
  `*EXPOS*`, `*BLOOM*`, `*COLOR*`, `*GAMMA*` (проверено grep по всему дереву).
- `crates/victory-hle/src/modules/display.rs`: HDR-интерфейс есть, но `try_enable_hdr` всегда отвечает
  «2» — в каждом прогоне лог пишет `HDR mode: disabled (SDR presentation) preference=0`.
- Презентация: `crates/victory-gpu/src/host.rs:8228-8273` создаёт swap chain **без `SetColorSpace1`**;
  в прогонах — `presenting to a window width=3840 height=2160 format=28` (DXGI 28 = `R8G8B8A8_UNORM`),
  при `VICTORY_DISPLAY_MODE=1920x1080`. То есть HDR-выхода нет вообще — это чистый SDR-путь.
- Наши прогоны §12/§15: мир 1,8–2,0 fps медиана (до прогрева) и **4,8–6,4 fps** после тёплой библиотеки
  пайплайнов; 0 ERROR; 6 отказов host-пайплайнов (`0x80070057`) на compute-шейдерах.

### 16.3. 254 непереведённых шейдера: точный разбор (`victory translate --list`)

Прогнал `victory translate C:\GTA6_Files\shaders.rpf --list --jobs 12` (релиз, 5 с). Итог: 26 084 программы,
**254 не переводятся** (полный список — `C:\GTA6_Port\recon\translate-failures.txt`):

| Причина | Штук | Что среди них (имена) |
|---|---|---|
| `resources: an address in an SGPR that is no root parameter` | **195** | все варианты `GS_VoxelizeGeneric_*` (слитые geometry-стадии; в README апстрима — «merged geometry stages, 195 programs») |
| `emit: a descriptor chosen by control flow` | **40** | `PS_TexturedDeferred_*`, `PS_TexturedLit_*`, `PS_MirrorReflectionTexturedDeferred_*`, `PS_TexturedLit_ReflectionMap_*`, `PS_PuddleDetect_*`, **`MST_DS_CascadeShadows_PNTriTexture`**, `MST_DS_TerrainTransform` |
| `emit: a hardware SGPR input` | 10 | `PS_PedTexturedCutOutTint_Zero_*` |
| `emit: a descriptor read only as data` | **5** | **`shadowmask:CS_TraceShadowRays_Eighth/Quarter/Half/Full/FullAdaptive`** |
| `emit: a lane mask holding lanes elsewhere read as data` | 2 | `water_shading:CS_TransmissionVolumetrics*` |
| `emit: an opaque lane mask` | 1 | `PS_VoxelizeVisiblity` |
| `emit: a lane mask exchanged where not every lane runs` | 1 | `cascadeshadows_rendering:CS_ShadowRenderTiledCascadesAlphaCBPPPC` |

**Прямой ответ по освещению:** шейдеры теней (`CS_TraceShadowRays` × 5, `MST_DS_CascadeShadows` × 2,
`CS_ShadowRenderTiledCascades*`), отражений (`*ReflectionMap`, `*MirrorReflection*`) и вокселизации
(`GS_VoxelizeGeneric` × 195) — **именно те 254, которые не переводятся**, их draw'ы бросаются. Это и есть
техническая причина «нет теней и непрямого света» — совпадает с `docs/fork-ray-tracing.md` (сборщик BVH
не запускается, 5 shadow-mask программ не переводятся) и с README апстрима («Missing: ray-traced shadows
and indirect light, the local-light shadow mask»).

**По HDR/тонемаппингу/экспозиции: среди 254 таких имён НЕТ** — ни одного `*Tone*`, `*Expos*`, `*Bloom*`,
`*Grade*`, `*Color*`. Значит пересвет нельзя списать на непереведённый шейдер HDR/экспозиции.

### 16.4. `fxceless/victory-fork` (наш «родственный» форк, самая свежая линия)

Состояние: default `main`, **1 ветка** (все работы — через ~100 открытых PR), pushed 2026-11-26 05:52 UTC;
за 200 последних коммитов **69 — после нашего снимка** (в основном диагностика и perf).

Что важно из свежего и из открытых PR (файлы проверены через `pulls/N/files`):

| PR/коммит | Тема | Файлы | Кэш шейдеров |
|---|---|---|---|
| **#1249** NVIDIA compatibility: Wave64 control flow + lossless d16 buffer accesses | **NVIDIA**: волновые 64-lane формы (наш случай — NVIDIA 32 lane!) и 16-битные буферы глубины | 12 файлов `victory-shader/src/emit/*` (lower, mod, halves, rect, loop_regions…) | **ДА** (меняет генерацию DXIL) |
| **#1166** Fix Windows shader warm-up, wave permutes, BVH residency | BVH-резидентность + прогрев, волновые перестановки | 5 файлов `victory-shader/emit` + `hle/d3d12x.rs`, `apps/victory` | **ДА** |
| **#1165** Skip ray-box children whose title-memory pages are missing | BVH: не читать отсутствующие страницы title-памяти | 4 файла `victory-shader/emit` (intersect, lower…) | **ДА** |
| #1217 hold ray walks after builder dispatch fails to bind | BVH-безопасность | 2 файла `victory-gpu` | нет |
| #1147 guards make only views read-only (мир не замерзал при builder on) | BVH-безопасность | 1 файл `victory-gpu` | нет |
| #1176 / #1248 wave-wide last resort / форма для 3 программ | 32-lane хост — последняя форма wave-wide | 1–2 файла `victory-shader/emit` | **ДА** |
| #1221 no-hit ray results, skip unused ray operands | BVH-корректность | 2 файла `victory-shader/emit` | **ДА** |
| #1195 null view of its class для слота без buffer view | шейдеры/GPU (meshlet-cull page faults) | 5 файлов `victory-shader/emit` | **ДА** |
| #1250 window budget, #1246 texture queue fairness, #1240 mip cap/RAM trim/descriptor bind cache, #1258 capped heap split | VRAM/стриминг/дескрипторы | только `victory-gpu` | нет |
| **#1224** `VICTORY_FILL_HOT`, `VICTORY_SHADOW_CACHE`, `VICTORY_BURY_ASIDE` (off) | 3 ручки frame-time | `victory-gpu` (2 файла) + README | нет |
| **#1178** lazy GPU write-back (VEH), O(dirty), AVX2 shadow | frame-time copy-back | 4 файла `victory-gpu` | нет |
| **#1227** send-and-hash list в 3 шага (29 → 6 мс card time/кадр); #1237 lookup cache (117 → 104 мс) | frame-time | по 1–2 файла `victory-gpu` | нет |
| #1233 AVX2 changed-runs, #1232 interval compaction, #1229/#1230 readback, #1234 pipeline wait once | CPU/frame-time | `victory-gpu` | нет |
| #1214 DRED-состояние при device removal, #1251 device-loss в тестах, #1228 CI | диагностика/CI | gpu + README | нет |
| #1255 (уже merged) | driver watch не считает **Auto HDR** notice ошибкой | `victory-gpu/drvwatch.rs` | нет |
| d4a528ba (уже в main) | текстуры по индексу может заливать отдельный поток (`VICTORY_HEAP_WORKER=1`) | gpu | нет |

Проверка по нашему дереву: символов/флагов `overwritten_movreld_edge`, `wave64_after_loop`,
`VICTORY_BVH_NO_HIT`, `VICTORY_HEAP_WORKER`, `VICTORY_LAZY_COPY_BACK`, `VICTORY_WRITE_PLAIN`,
`VICTORY_TEXTURE_DROP_LEVELS`, `VICTORY_PRESENT_QUEUED`, `VICTORY_FIDELITYFX_DIR` **нет** — всё это
кандидаты, которых у нас пока нет.

### 16.5. `Marvisak/VIctory` (общий апстрим)

Открытых PR — ~100, открытых issue — ~200 (выборка по 100/200). Самое релевантное:

| Issue/PR | Что | Файлы | Кэш |
|---|---|---|---|
| **commit `f342dcfcfb`** «Inputs a counted v_movreld loop overwrites first need no value» — прямо про **`PS_DownSampleBloomFilter(Karis)_557ae3d7`** (bloom downsample, 3 draw/кадр в интро) | возможный источник **пересвета bloom**: входы цикла перезаписывались раньше чтения | `victory-shader/src/emit/lower.rs` (+33 КБ диффa) | **ДА** |
| **#2268** «three user-data corrections clear the last address class (**195 → 0**), mask reuse exposes gap (298 → 103)» (has patch) | лечит ровно наш класс `address in an SGPR` — **195 шейдеров вокселизации** | `victory-shader` (resource/emit) | **ДА** |
| **#2750** «GPU/BVH: DispatchX ignores flag 0x20 — thread-count vs group-count mismatch in builder dispatches» (has patch) | почему BVH-сборщик не строит | `victory-gpu` (indirect) | нет |
| **#2368** «root-derived BVH node addresses lose high pointer bits and descriptor alignment» (confirmed) | BVH-адресация | `victory-shader`/gpu | **ДА** |
| **#2045** «Fog apply pass empties the lighting target: CS_ApplyCombinedDeferred множит на входы без писателя» | освещение/туман: на main 940d32f неисправен сам apply-pass | `victory-shader`/gpu | **ДА** |
| **#841** «prologue рисует в 2560x1440, а 3840x2160 выход остаётся нулевым» (RTX 3090) | наш случай: заголовок отдаёт **4K output** (format 28), при этом рендер 1440p — вопрос upscale/композита | gpu | нет |
| **#2960** async submission + copy queue (17 с → 0.3–2 с/кадр, порт из fxceless) | FPS: в нашем дереве `VICTORY_ASYNC_SUBMIT` **уже есть** | gpu | нет |
| **#2958** mark-lead: фреймы **411 → 283 мс** на RTX 4070 SUPER | FPS/CPU | `victory-shader/emit` (mark_view_*) | **ДА** |
| **#3011** thin LTO, x86-64-v3, xxh3 для текстур | FPS/build, поведение не меняет | Cargo.toml, gpu/hle | нет |
| **#2993** host copies в VRAM под DXGI-бюджет (ReBAR, opt-in) | VRAM/FPS | gpu | нет |
| **#3031** shadow copy-back stale fix; **#2962** RGB10 проба чёрного кадра; **#2602** 165 непереводимых → 2534 draw/frame; **#255** 250 indirect-rectangle draw байпасится | мелочи/диагностика | gpu | #3031 нет, #2962 нет |
| **#2052** «preserve NaN в HDR-диагностических захватах» | только диагностика захватов | gpu | нет |

### 16.6. `xcl-dev/xcl` (это НЕ форк VIctory — отдельный слой, GPL-2.0)

- `xcl-dev/xcl` — самостоятельная реализация того же класса («Xbox Series compatibility layer… recompiles
  RDNA2 shaders to DXIL»), **не форк** `Marvisak/VIctory`; README fxceless прямо пишет, что XCL (ivall/xcl)
  использован как **референс**, а не как код. Поэтому **прямого переноса нет** — только идеи.
- 4 ветки: `main`, `amd-dev`, `nvidia-dev`, `shader-prep`; ~100 открытых PR.
- **PR #2057** `integration/nvidia-world`: «D3D12.X: integration branch that reaches the in-world HUD on
  NVIDIA (reference, not for merge)», base `main`, 87 коммитов, **48 файлов, +9547/−1149**.
  Внутри — NVIDIA-путь: `d3d12x/{real,sparse,title_textures,texture_upload,gpu_profile,indirect_guard,
  look_stats,shard,worker_pool}`, `gpu/recomp/*`, `gpu/tiling.rs`, `gpu/uma.rs`, `hle/dstorage`.
- Замеры на RTX 5090 (driver 621.14) с `XCL_DIRECT_DXIL=1 XCL_HEAP_WINDOWS=1 XCL_UMA_SHADOW=1
  XCL_COHERENT_HEAPS=sparse:16 XCL_LOOK_SKIP=1`: меню/интро 30 fps, мир через 86–104 с, **5–10 fps**
  (2 000–2 446 кадров); отмечено, что «14 больших graphics-пайплайнов компилировались 5–7 с каждый» — тот же
  класс пайплайн-шторма, что мы разбирали в §14.
- **HDR/тонемаппинга в PR #2057 и в поиске по репозиторию нет** (поиск `HDR OR tonemap OR exposure OR bloom`
  — 0 результатов). Оптимизация — за счёт D3D12.X-слоя и шейдеров, не цвета. Вывод: по пересвету xcl
  ничего готового не даёт; по FPS — подтверждает, что на NVIDIA тот же титул другим слоем идёт 5–10 fps.

### 16.7. Выводы по трём симптомам

1. **Пересвет (HDR/Bloom)** — HDR-выхода в нашем слое нет и не планируется (HDR API отвечает «disabled»,
   swap chain SDR RGBA8 без ColorSpace); среди 254 непереведённых шейдеров **нет** тонемаппинга/экспозиции/
   bloom. Основные версии, по убыванию правдоподобия:
   (а) **следствие отсутствия теней/непрямого света**: прямой свет без окклюзии (lamp/sun shadow mask не
   запускается) + bloom по несжатым HDR-значениям → выжигание лампочек/окон/спины, а комната без bounce — тёмная;
   (б) **bloom-шейдер с ошибкой**: в апстриме ровно `PS_DownSampleBloomFilter(Karis)` чинили коммитом
   `f342dcfcfb` (наш снимок его не содержит — символов нет), а bloom в интро — 3 draw/кадр;
   (в) **OS-side** Auto HDR / NVIDIA RTX HDR поверх SDR-окна (наш слой показывает SDR; в апстриме есть
   даже отдельная правка «driver watch не считает Auto HDR notice ошибкой», #1255) — это надо выключить и
   проверить, самый дешёвый тест.
2. **Нет непрямого света и теней** — доказанная в коде причина: RT-сборщик BVH не запускается (64-lane
   пайплайны и 5 shadow-mask программ не переводятся), 40 «descriptor by control flow» включают
   cascade-shadow и reflection PS, 195 вокселизации не переводятся; их draw'ы бросаются. Лечится портом
   BVH/волновых фиксов (#1147/#1165/#1217/#1221/#1249/#1166 + issue #2750/#2368/#2268).
3. **FPS 1–3 и мыльные текстуры** — это наш известный профиль: мир 1,8–2,0 fps на холодную и 4,8–6,4 fps
   на тёплую библиотеку (§12/§15); в живом геймплее тяжелее — 1–3 fps, как и у пользователя. «Мыло» —
   очередь текстур (16k ждут/несмапленные, §11/§12) + 254 брошенных draw. Быстрые кандидаты — perf-PR
   fxceless (все без кэша) + #3011 (LTO/x86-64-v3) + #2993 VRAM-копии.

### 16.8. Что предлагаю взять (после вашего решения; пока код не менял)

Порядок по соотношению «эффект/риск», кэш шейдеров указан у каждого:

1. **Бесплатный тест без кода**: выключить Windows Auto HDR и NVIDIA RTX HDR, повторить кадр в тюрьме —
   проверить версию 1(в). Плюс `VICTORY_D3D12_DRED=1` если поймаем `0x887A0005`.
2. **Bloom/movreld-фикс `f342dcfcfb`** (Marvisak main) — точечный порт в `emit/lower.rs`, **кэш ДА**
   (ключ кэша содержит хэш исходников транслятора, `VICTORY_SHADER_SOURCE_HASH` из `emit/mod.rs:201`) →
   после порта обязательный `translate --warm all`; риск низкий; может снять выжигание bloom.
3. **#2268 (три user-data правки)** — переводит 195 программ вокселизации (GI/слитые стадии), **кэш ДА**;
   самый большой «световой» выигрыш из дешёвых.
4. **NVIDIA wave64: #1249 + #1176/#1248** — 12 файлов эмиттера, **кэш ДА**; лечит 6 отказов compute-пайплайнов
   (`0x80070057`) и часть 64-lane форм; средний риск, обязательны `cargo test --workspace` и перегрев.
5. **BVH-путь теней**: #1147 + #1217 + #1250 (gpu, без кэша) → затем #1165/#1221/#1166 (кэш ДА) + #2750/#2368
   из апстрима. Даёт ray-traced тени/непрямой свет, но исторически морозил мир — только пошагово, с DRED.
6. **Perf-серия fxceless (всё без кэша)**: #1227 (29→6 мс card time), #1237 (117→104 мс), #1224 (3 ручки),
   #1178 lazy write-back, #1240 mip cap/RAM trim, #1233/#1232/#1229/#1230 — батчем, с A/B как в §15.
7. **#3011** LTO/x86-64-v3/xxh3 — build-only, кэш нет; +`VICTORY_HEAP_WORKER=1` из свежего main fxceless.

Правило по кэшу: всё, что меняет `crates/victory-shader` (эмиттер/лоуэринг/ресурсы), меняет и хэш исходников
транслятора в ключе кэша → прогретый кэш (25 635 переводов) **автоматически перестаёт использоваться**;
после порта нужен полный `translate --warm all` (без карты — ~70–90 с на 12 потоках). Всё, что только
`victory-gpu`/`victory-hle`, кэш не трогает.

### 16.9. Файлы этой сессии
- `C:\GTA6_Port\recon\` — `fx-commits.txt`, `fx-commits-1/2.json`, `fx-branches.txt`, `fx-prs.txt/.json`,
  `fx-README.md`, `fx-pr-NN*.json` (файлы отобранных PR), `mv-commits.txt`, `mv-issues.txt`, `mv-prs.txt`,
  `mv-README.md`, `mv-issue-NN.json`, `mv-bloom-fix.diff`, `mv-hdr-issues.json`, `mv-removed-issues.json`,
  `xcl-meta.json`, `xcl-commits.txt`, `xcl-branches.txt`, `xcl-prs.txt`, `xcl-pr2057*.json`,
  `translate-list.txt`, `translate-failures.txt`, `nonvoxel-failures.txt`;
- скрипты разведки: `C:\GTA6_Port\recon\recon-fx.ps1`, `recon2.ps1`, `recon-prs.ps1`;
- код не менялся; `git status` — только `AGENTS.md`/`HANDOFF.md` (теперь закоммичены).

## §17. Ветка `visual-and-nvidia-fixes`: перенос #2268 и NVIDIA-слоя (2026-11-26)

Задание: починить пересвет (кандидат 1 = bloom `f342dcfcfb`), вокселизацию/свет в помещениях
(кандидат 2 = #2268) и совместимость с NVIDIA (кандидат 3 = #1249/#1176/#1248). Сначала три параллельных
субагента-разведчика (каждый сохранял диффы в `C:\GTA6_Port\upstream\`), затем главный агент вносил правки
последовательно с `cargo check --workspace` после каждого шага. Ветка создана от `perf-and-upstream-fixes`.

### 17.1. Итоги разведки субагентов (главное)

| Кандидат | Что нашли | Решение |
|---|---|---|
| **Bloom, Marvisak `f342dcfcfb`** | Коммит — ручной порт открытого PR #457; чинит отказ `barycentrics read directly` у `PS_DownSampleBloomFilter(Karis)`. **Но наша линия другая**: наш `lower.rs` = fxeless-линия с `lazy_barycentric` (barycentric-входы грузятся как `SV_Barycentrics`), такого отказа у нас нет; bloom-шейдер **уже переводится**; в списке 254 его нет. Порт был бы мёртвым кодом + 7 минут перегрева кэша. | **Не переносил** (фальсифицирован). Патч-кандидат сохранён: `upstream\marvisak-bloom-port-adapted.diff` (+9 КБ, применим, поведение не меняет) — если захотите эмпирически проверить. Пересвет — из §16.7 (а)/(в): нет теней/GI и/или OS Auto HDR. |
| **#2268, Marvisak** | PR №2268 **не существует**; это issue с патчем в ветке форка `bittertea0323/VIctory:try/upstream-base`, коммиты `a35b26d` и `5eaf119` (итого +26/−8 в `native.rs`/`resource.rs`). Upstream его **отверг** (#2546 закрыт: «weakens a soundness check»; #2284 — только тесты). Патч адаптирован к нашему дереву (`upstream\marvisak-2268-adapted-to-victory-fork.diff`, `git apply --check` OK). | **Перенесён** (2 файла, +26/−8). |
| **NVIDIA, fxceless #1249/#1176/#1248** | #1176 — 1 файл `emit/mod.rs` (+23 строки): wave-wide форма как last resort; APPLIES целиком. #1249 — 19 файлов, из них три слоя: (A) Wave64 control-flow для 32-lane хоста (loop_regions + lower.rs + 1 строка mod.rs), (B) lossless d16, (C) R11-эксперимент «superseded». #1248 — альтернатива #1176 (селективный список + отдельный ключ кэша), логически конфликтует с #1176. | **Перенесены #1176 + слой A из #1249**; слой B (d16) и #1248/C — отложены (см. 17.5). |

### 17.2. Что изменено в коде (5 файлов, +77/−23 + новый файл)

- `crates/victory-shader/src/native.rs` (+2/−2): `Gs` — user data с s0 (standalone), а не с s8.
- `crates/victory-shader/src/resource.rs` (+24/−6): `param_of` пробует сдвиги `[0,8]` для GS; high half
  проверяется только у root-дескриптора (`high.is_some()`), у таблиц — нет.
- `crates/victory-shader/src/emit/mod.rs` (+25/−1): блок «wave-wide branches (last resort)» (#1176) после
  всех форм; `branches_around` получает третий аргумент (`native.wave64 && host_max_wave()==32`).
- `crates/victory-shader/src/emit/lower.rs` (+37/−4): `all_lane_blocks/lane_regions/branches_around` c
  `loop_exits`; учёт natural loops при выходе из цикла; 4 call-site (LDS-проверки — с `true`; barrier/wave-sum —
  только для `Cs && wave64 && host_max_wave()==32`).
- `crates/victory-shader/src/emit/loop_regions.rs` — **новый** (51 строка): `natural_loops` CFG.
- `crates/victory-shader/src/emit/wave64_tests.rs` (+2/−2): два assert под новую сигнатуру.

Каждый шаг: `cargo check --workspace` → **exit 0** (сначала после #2268, затем после NVIDIA-слоя).

### 17.3. Проверки, сборка, прогрев

- `cargo test --workspace` — **зелёный с первого прогона, exit 0**: core 11, formats 25 (+4 ignored),
  gpu 184 (+6), hle 100, loader 27+2+1, pe 7, pipeline 4, shader 216 (+3), xtask 4, doc 0. Флаки warp-тестов
  в этот раз не проявились.
- `cargo build --release -p victory` — **exit 0, 43,03 с**; `target\release\victory.exe` 8 501 248 Б, 12:00:52.
- Новый ключ кэша (хэш исходников `victory-shader` изменился) → полный прогрев той самой командой:
  `$env:VICTORY_HOST_WAVE='32'; $env:VICTORY_VIEW_LIMIT='999999';
  .\target\release\victory.exe translate 'C:\GTA6_Files\shaders.rpf' --warm all --jobs 12`
  → **249,4 с**, `all hashes asked, 25889 found: 25637 translated now, 0 in the cache already,
  252 do not translate`.

### 17.4. Сколько из 254 перевелось (главный результат)

Сравнение `translate --list` до/после (полные списки: `C:\GTA6_Port\upstream\translate-failures-BEFORE.txt`
и `C:\GTA6_Port\recon\translate-list-AFTER.txt`):

| | Было (§16) | Стало |
|---|---|---|
| Непереведённых программ (`--list`) | **254** | **57** |
| Из них исправлено | — | **197** (195 `GS_VoxelizeGeneric_*` + 2 `water_shading:CS_TransmissionVolumetrics*`) |
| Новых падений | — | **0** |
| DXIL compute emitted | 2674 / 2682 | **2676 / 2682** (+2) |
| Группа `resources: an address in an SGPR…` | 195 | **0** |
| Осталось: `descriptor chosen by control flow` (cascade shadows, reflection PS, puddles) | 40 | 40 |
| Осталось: `hardware SGPR input` (`PS_PedTexturedCutOutTint_Zero`) | 10 | 10 |
| Осталось: `descriptor read only as data` (**5 × `CS_TraceShadowRays`**) | 5 | 5 |
| Осталось: `opaque lane mask` / `lane mask exchanged` | 1 / 1 | 1 / 1 |

**Почему `--warm all` показывает 252, а не 57** (честное объяснение): счётчик прогрева идёт через
`victory_gpu::shader_cache::warm` → `victory_shader::emit::translate`, который **не эмитит GS-стадию**
(merged geometry stages; в отчёте `--list` есть только VS/PS/CS/MS). 195 GS-программ после #2268 успешно
проходят resource-tracking (поэтому исчезли из `--list`), но DXIL для GS-стадии у нас всё равно не
генерируется — и прогреватель считает их «do not translate» (195 + 57 = 252). Реальных новых переводов
в DXIL ровно **+2** (две водные compute-программы, слой A #1249). Практический вывод: #2268 улучшает
метрику трассировки ресурсов и предпосылку для будущей GS-эмиссии, но **картинку сейчас не меняет**.

### 17.5. Что НЕ взято и почему (риски)

1. **Bloom `f342dcfcfb` — не переносился**: для нашей линии no-op (см. 17.1). Пересвет лечится не им.
2. **#1249 слой B (lossless d16)** — не входит в 6 отказов compute; трогает GPU-крейт в местах, где наша
   линия уже разошлась (`descriptor/host.rs` и `draw.rs` — конфликт), требует своего цикла проверки.
   Отложено отдельной задачей.
3. **#1248** — альтернатива #1176; его селективный список конфликтует с уже применённым last-resort
   (он всё равно ловит исключённые хеши). Оставлено на случай, если #1176 даст артефакты.
4. **#1176 — blanket last-resort**: автор #1248 предупреждал, что wave-wide для `369cbe906ea72764`
   (`GpuParticleUpdate:CS_Visibility`) и `5dc3b259`/`6f225b64` (water volumetrics) давал падение карты /
   чёрные пятна на воде. Теперь эти формы будут пробоваться в последнюю очередь. **Откат без пересборки:
   `VICTORY_WAVE_WIDE=0`.** Это главный риск шага — проверять игровым прогоном.
5. **Оставшиеся 2 compute-отказа** (`2da4beb3`, `02384a60`) этими PR не лечатся — нужен #1124
   (per-instruction EXEC guards), вне текущего задания.
6. **#2268 отвергнут upstream'ом** по семантике (standalone-GS раскладки); у нас он adopted как
   tracking-only. Возможен откат `git revert` этого файла, если понадобится.

### 17.6. Следующий шаг (по заданию — стоп перед perf-батчем)

1. Игровой прогон на новой сборке: проверить пересвет (стало ли лучше/иначе), воду/партиклы (риск #1176)
   и 6 отказов compute в логе (ожидание: `2cc4c19` и, возможно, `369cbe9`/водные — с пайплайном;
   `2da4beb`/`02384a6` — останутся).
2. Если артефакты — `VICTORY_WAVE_WIDE=0` как откат без пересборки.
3. Затем perf-батч (кандидат 5 из §16.8): #1227, #1237, #1224, #1178, #1240, #1233/#1232 + #3011.
4. Для теней/непрямого света остаются кандидаты 4 (BVH-путь) и 5×`CS_TraceShadowRays` (порт нужен
   из будущего апстрима — в текущем срезе они не переводятся).

### 17.7. Файлы и состояние

- Ветка `visual-and-nvidia-fixes` (от `perf-and-upstream-fixes`), коммит с этим отчётом запушен в `myfork`.
- Диффы и разборы: `C:\GTA6_Port\upstream\marvisak-2268-*.diff`, `marvisak-2268-analysis.md`,
  `marvisak-bloom-port-adapted.diff`, `fxceless-pr1249.diff`, `fxceless-pr1176.diff`,
  `fxceless-pr1248.diff`, `pr1249-parts\*`, `fxceless-pr*-apply-check.txt`;
  результаты `--list`: `C:\GTA6_Port\recon\translate-list-AFTER.txt`, лог прогрева
  `C:\GTA6_Port\recon\warm-after-visual.log`;
- `git status` после коммита чист; `master` не тронут.

## §18. Мониторинг звёзд и perf-батч без сброса кэша (2026-11-26, ветка `visual-and-nvidia-fixes`)

Задание: перед накатом perf-батча заново прочекать ВСЕ репозитории в звёздах `4EKUSHKAq` на свежее за
последние 24–48 ч (коммиты, ветки, PR), тремя параллельными субагентами разобрать perf-PR
(#1237/#1227/#1178/#1224) и VRAM-PR (#1240/#1258), затем главным агентом вшить совместимые фиксы,
**не трогая `victory-shader`** (прогретый кэш 25 637 шейдеров должен остаться валидным), проверить
`check`/`test`, собрать релиз, закоммитить, запушить и обновить HANDOFF.

### 18.1. Мониторинг звёзд (субагент stars-monitor)

Звёзд всего **6**; реальная активность за 48 ч (окно `2026-11-24T07:29Z → 2026-11-26T07:29Z`) — в 5.
Итого 2085 коммитов и 5849 PR. `ivall/xcl` не существует отдельно — GitHub редиректит на `xcl-dev/xcl`.

| Репозиторий | Коммитов 48 ч | PR: создано / merged | Что важно |
|---|---|---|---|
| `fxceless/victory-fork` (наша линия) | 167 | 1009 / 7 | вся работа через PR; **54 коммита и 45 новых PR после 00:00Z**, новейший **#1269** (07:15Z) — «больше места под текстуры на 8 ГиБ» |
| `avatarmanz/VIctory-7900XTX` | 518 | 69 / 0 | репо создан 25.11 22:18Z; **стенд-ин непрямого света** (`VICTORY_AMBIENT`, новый `gpu/ambient.rs`), адрес view по индексу (`027e367af` — 4 из 5 `CS_TraceShadowRays`), маски над «склеенным числом» (`d1f883ecb` — последний трейсер); всё это **меняет шейдеры** |
| `Marvisak/VIctory` (апстрим) | 486 | 2152 / 269 | вся активность — залп мержей 21:52–21:57Z 25.11; **он уже в нашем снимке** (`VICTORY_DEPTH_TWINS` #2162 и др. есть) |
| `uiliannn/VItoriaBr` | 231 | 101 / 29 | сводные порты; автогвардинг дивергентных single-wave64 барьеров на 32-lane (NVIDIA) — близко к уже взятому #1176 |
| `xcl-dev/xcl` | 683 | 1772 / 295 | тишина с 00:48Z; GPL-2.0, **не форк** — только идеи, код брать нельзя |
| `Flowseal/zapret-discord-youtube` | 0 | — | попал в окно только по `updated_at` |

Граница нашего снимка: в нём есть `VICTORY_DEPTH_TWINS` (#2162) и `VICTORY_SHADER_OPT`/`VICTORY_BACK_READS`,
но нет `VICTORY_BVH_NO_HIT` из fxceless `96ff13566` (00:00:13Z 26.11) → **наш срез ≈ 25.11 22:00Z**,
отставание от fxceless main ≈ 100+ коммитов (это и есть причина конфликтов свежих PR).
Темы A (свет/тени): готовых фиксов HDR/tonemap/exposure/bloom за 48 ч **нет нигде** (только диагностика
NaN в захватах #2052 и Auto HDR notice #1255); вся новизна — ray-traced свет/BVH и шейдерные порты.
Тема B (FPS): почти всё измеренное — те самые PR, что в батче. Тема C (8 ГиБ/NVIDIA): #1269/#1250/#1258/
#1240/#1219/#1218 + `VICTORY_SHOW_NEW_LATER` (avatarmanz `4cd2d1335`, device loss каждые ~15 мин).

### 18.2. Что вшито (6 шагов, после каждого `cargo check --workspace` → exit 0)

| Шаг | Что | Файлы / строки | Флаги (по умолчанию) |
|---|---|---|---|
| 1 | **#1237 + #1245** (upstream main `32be36d5`+`bc6ee68e`): кэш поиска host-копий на поток | `shadow.rs`, `host.rs` +223/−15 | `VICTORY_LOOKUP_CACHE` (**выкл**; `=1` вкл, `=alt` A/B 20 с) |
| 2 | **#1227** (upstream `6aed0780`): send-and-hash в 3 шага, 29 → 6 мс card time | `host.rs` +186/−34 | `VICTORY_CARD_SYNC_BATCH` (**выкл**; `=1`, `=alt`) |
| 3 | **#1224, только host-часть**: горячие fill'ы и «убрать в сторону» | `host.rs` +126/−6 | `VICTORY_FILL_HOT=1`, `VICTORY_BURY_ASIDE=1` (**выкл**) |
| 4 | **#1258 + #1240** (combined): capped heap split + mip cap / RAM trim / bind cache / батч-зеркало дескрипторов | `descriptor/host.rs`, `descriptor.rs`, `draw.rs`, `host.rs`, `list.rs`, `resource.rs`, `shadow.rs`, `texture.rs` | `VICTORY_DYNAMIC_SLOTS` (авто: 393 215 title + 606 784 dynamic), `VICTORY_TEXTURE_MAX_DIM` (выкл, ≥64, нужен `LEVELS_ON_DEMAND=1`), `VICTORY_RAM_TRIM_MB` (выкл), `VICTORY_BIND_CACHE` (выкл); всегда-вкл. zero-page/slot skip, batch-mirror, memset-skip (откат `VICTORY_HEAP_SYNC_PAGE=0`) |
| 5 | **#1269** (свежий PR 07:15Z, ручной порт 20 строк): на картах ≤8 ГиБ резерв под текстуры 10 % → 6 % | `resource.rs` +47/−8 | без флага: `VICTORY_TEXTURE_ROOM_PERCENT` по умолчанию **94** на ≤8 ГиБ, 90 на остальных; явная переменная всё ещё главнее |
| 6 | README-блоки #1224 | `README.md` +14 | — |

Итого дифф: **9 файлов, +1124/−81** (без HANDOFF). `crates/victory-shader` — **0 изменений** (проверено
`git diff --stat -- crates/victory-shader` = пусто), поэтому ключ кэша не изменился.
Новых тестов от PR: `capped_dynamic_slots`, `records_copied_as_bytes_are_made_at_the_next_sync`,
`two_dirty_card_buffers_reach_readback_in_the_same_submission` (ignored, нужен адаптер), наш
`low_vram_texture_room_default_is_higher`.

### 18.3. Что НЕ взято и почему

1. **#1178 (lazy GPU write-back)** — 6939 строк в 4 файлах, зависит от **14 upstream-коммитов после нашего
   среза** (`6d20eed6`/`94ed363c`/`de4146a0`/`84c5dc00`/`1391409c`…), 3-way merge даёт 10 конфликтов, и он
   переписывает `find_all`/`find_pieces`, где у нас уже живёт свой `VICTORY_PREFETCH` (§15). Механически
   не собирается; нужен либо rebase автором на наш срез, либо отдельный порт «upstream main 44bff61b→dc4c0fcd».
2. **#1224 shadow-кэш** (вторая половина PR) — дубль по смыслу с #1237 и текстовая несовместимость
   (`shadow::lookup`/`BUFFERS` переписываются двумя разными кэшами); A/B автора нейтрален (6,60→6,61 fps).
3. **#1226** (`translate --cache --pipelines`, 15 файлов, +2728) — самая интересная идея против нашего
   pipeline-шторма (530–549 PSO в кадре, §14), но это отдельная задача: трогает `apps/victory`,
   `Cargo.toml`/`Cargo.lock` и требует своей проверки.
4. **#1250 / #1219 / #1218** (VRAM-окно, `VICTORY_TEXTURE_DROP_LEVELS`, hold reads) — по смыслу полезны,
   но конфликтуют с нашим деревом (свежий base); нужна ручная адаптация, отложены.
5. **Шейдерные порты по свету** (`avatarmanz 027e367af`/`d1f883ecb` на 5 `CS_TraceShadowRays`,
   `uiliannn` #34/#79, fxceless #1263) — **меняют `victory-shader`** → сброс кэша и полный `--warm all`
   (~250 с). Отдельная задача, как и `VICTORY_AMBIENT` (стенд-ин GI из другой линии форков).
6. **Кэш по `TRANSLATOR_VERSION`** (avatarmanz `1162d38f9`) — убрал бы 250-с прогрев на каждый порт
   эмиттера, но сам меняет ключ кэша; решение за пользователем.

### 18.4. Проверки, сборка, кэш

- `cargo check --workspace` — после каждого из 6 шагов **exit 0** (логи: `upstream\check-s1..s5*.log`).
- `cargo test --workspace` — **exit 0**: core 11, formats 25 (+4 ignored), gpu **188** (+7 ignored; было 184/+6),
  hle 100, loader 27+2+1, pe 7, pipeline 4, shader 216 (+3 ignored), xtask 4, doc 0. Наш новый тест
  `resource::tests::low_vram_texture_room_default_is_higher ... ok`.
- `cargo build --release -p victory` — **exit 0, 30,88 с**; `victory.exe` 8 592 384 Б, 12:52:25.
- **Кэш шейдеров не сброшен** (главное требование): обычная команда прогрева вернула
  `all hashes asked, 25889 found: 0 translated now, 25637 in the cache already, 252 do not translate;
  4,7 s on 12 threads` — т.е. перепрогрев не нужен, все 25 637 переводов на месте.

### 18.5. Итоговая команда запуска (PowerShell, из корня `C:\GTA6_Port\victory-fork`)

```powershell
$env:VICTORY_DISPLAY_MODE='1920x1080'; $env:VICTORY_LEVELS_ON_DEMAND='1'; $env:VICTORY_ASYNC_SUBMIT='1'; $env:VICTORY_QUEUE2='1'; $env:VICTORY_INDIRECT_WINDOW='1'; $env:VICTORY_TEXTURE_NO_WAIT='1'; $env:VICTORY_MERGE_GAP='64'; $env:VICTORY_MERGE_RUNS='4096'; $env:VICTORY_STAGING_POOL='0'; $env:VICTORY_COPY_PLAIN='1'; $env:VICTORY_JOIN_PARTS='1'; $env:VICTORY_BACK_OWN_VIEW='1'; $env:VICTORY_INDIRECT_GATHER='1'; $env:VICTORY_CARD_REHASH_RUNS='1'; $env:VICTORY_HOST_WAVE='32'; $env:VICTORY_VIEW_LIMIT='999999'; $env:VICTORY_LOOKUP_CACHE='1'; $env:VICTORY_CARD_SYNC_BATCH='1'; $env:VICTORY_FILL_HOT='1'; $env:VICTORY_BURY_ASIDE='1'; $env:VICTORY_BIND_CACHE='1'; $env:VICTORY_TEXTURE_MAX_DIM='1024'; .\target\release\victory.exe run 'C:\GTA6_Files\game_scarlett_master.exe' --execute
```

Что нового в команде против §8: `VICTORY_LOOKUP_CACHE=1` (117→104 мс на замере автора),
`VICTORY_CARD_SYNC_BATCH=1` (29→6 мс card time), `VICTORY_FILL_HOT=1` + `VICTORY_BURY_ASIDE=1`
(ручки frame-time), `VICTORY_BIND_CACHE=1` (CPU/кадр), `VICTORY_TEXTURE_MAX_DIM=1024` (главный рычаг
VRAM — режет старшие mip'ы; при всё ещё >95 % VRAM ставить `768`), `VICTORY_TEXTURE_ROOM_PERCENT`
**не задаём** — теперь автоматически 94 % на нашей 3060 Ti. `VICTORY_DYNAMIC_SLOTS` тоже не задаём:
раскладка «393 215 слотов title + 606 784 dynamic» выбирается сама. `VICTORY_RAM_TRIM_MB` — опционально
(у нас 32 ГБ ОЗУ, это про ОЗУ, не про VRAM).
Честный A/B (если что-то пойдёт не так): у `VICTORY_LOOKUP_CACHE` и `VICTORY_CARD_SYNC_BATCH` есть
`=alt` (20 с вкл / 20 с выкл с отметкой в логе) или `=0`.

### 18.6. Риски и что смотреть в прогоне

1. **#1258 (heap split)** — title-слоты 393 215…1 245 183 теперь читают dynamic-слоты; если там окажется
   долгоживущий view, выбранный по индексу, прочитается null (автор таких не видел). В логе ждать
   `host view heap capped at 1000000` с `fitted=393215`, следить за «no free dynamic descriptor slots».
2. **#1240** — cap 1024 может «замылить» дальние текстуры (это компромисс за VRAM); batch-зеркало
   дескрипторов — откат `VICTORY_HEAP_SYNC_PAGE=0`.
3. Все новые флаги — **opt-in и на 3060 Ti ещё не измерялись** (замеры авторов: RTX 3070 8 ГиБ, 5060 Ti).
   При артефактах снимать по одному, в порядке: `VICTORY_BURY_ASIDE`, `VICTORY_FILL_HOT`,
   `VICTORY_CARD_SYNC_BATCH`, `VICTORY_LOOKUP_CACHE`.
4. Ожидание по VRAM: пик был 6802/7104 МБ (96 %), 206 перезаливов текстур; с cap 1024 и 94 % под текстуры
   ждём заметно ниже и меньше «the card has no room for more».
5. Кэш шейдеров перегревать **не нужно** — если после правок `victory-shader` кто-то будет портировать
   шейдерные фиксы, тогда да: полный `--warm all`.

### 18.7. Файлы

- Патчи и разборы: `C:\GTA6_Port\upstream\perf\` (`01-pr1237-safe.diff`, `02-pr1227-main.diff`,
  `03-pr1224-host.diff`, `05-pr1224-README.diff`, `00-all-three-combined.diff`, `REPORT.md`, `check-*.txt`,
  `scratch\`), `C:\GTA6_Port\upstream\vram\` (`pr1258+pr1240.combined.diff`, `pr1240.diff`, `pr1258.diff`,
  `ANALYSIS.md`, `apply-check.txt`), `C:\GTA6_Port\upstream\fresh\` (`pr1269.diff` + метаданные #1269/#1250/
  #1245/#1226/#1219/#1218);
- Сырые данные мониторинга: `C:\GTA6_Port\recon\stars\` (`stars-report.md`, `commits-48h.tsv`,
  `pr-merged-48h.tsv`, `pr-open-48h.tsv`, `pulls-all.tsv`, `topic-A/B/C-*.txt`, `commits-shader-path.tsv`);
- Логи: `upstream\check-s1..s5*.log`, `upstream\test-after-perf.log`, `upstream\build-after-perf.log`.

## §19. Публичный форк `codec-d/cool_fox` и ветка `coolfox-rtx` (2026-11-26)

Задание: подключить `https://github.com/codec-d/cool_fox.git` (якобы «топовый форк на базе fxceless»),
тремя параллельными субагентами разобрать лаунчер/пресет `pareto-residency`, перф-коммиты и свет,
затем главным агентом сравнить **Путь А** (ветка от `coolfox/main` + наши файлы) и **Путь Б** (перенос
их коммитов в нашу ветку), выбрать надёжный, собрать, прогреть кэш, запушить в `myfork` и дать команду.

### 19.1. Что такое cool_fox на самом деле

| Факт | Значение |
|---|---|
| Метаданные | создан 2026-11-26 04:16Z, `isFork=false`, **667★ / 188 форков**, 1 ветка `main`, 1 тег `pareto-residency-37a410b1` = `f4b48ad` |
| История | **627 коммитов, отдельная** (`git merge-base HEAD coolfox/main` пуст) → cherry-pick невозможен, только сравнение содержимого |
| Происхождение | форк `avatarmanz/VIctory-7900XTX` (тот же репозиторий из §18.1), который сам — линия fxceless; сверху сведены Marvisak/Seenax/uwucammie/DexNom/Belobug/romainhedouin |
| Дерево | 250 файлов, полноценный workspace (`crates/*`, `apps/*`, `xtask`), **бинарников в git нет**; toolchain `1.98.1` msvc — совпадает с нашей машиной |
| Лаунчеры | `Play-VIctory.cmd` (41 строка, только аргумент `pareto-residency`) и `scripts/play.ps1` (`-Safe -RayTracing -Build`) |
| Безопасность | **чисто**: 0 совпадений по `Invoke-WebRequest/BitsTransfer/DownloadString/bitsadmin/certutil/schtasks/HKLM/HKCU/FromBase64String/EncodedCommand`, `https?://` в скриптах 0; только `set "VICTORY_*=..."` + запуск локального exe |

Важно: **заявления «~25 FPS на NVIDIA» в репозитории нет**. Их же замеры: RX 7900 XTX 24 ГиБ — цель
«30 fps, пока не достигнута», 13–22 fps в прологе; **RTX 5090 / 32 ГБ RAM** — медиана `54.3 мс` (~18,4 fps),
лучшие интервалы ~20 fps; `README-fxeless` про **RTX 4070 12 ГБ** — «not playable yet, **2.5–4 fps**».
Т.е. цифру 25 нельзя переносить на нашу 3060 Ti даже как ориентир; зато их «32 GB RAM residency budgets»
(коммит `8cc31ab`) — ровно наш профиль (32 ГБ ОЗУ).

### 19.2. Пресет `pareto-residency` (то, что реально ставит лаунчер)

Раннер: `victory-3d-rtx-pareto-residency.exe` в корне, иначе `target\release\victory.exe`.
Команда: `run "<VICTORY_GAME_DIR>\game_scarlett_master.exe" --console series-s --execute`.
Все присваивания — `if not defined`, т.е. наш env имеет приоритет.

| Группа | Переменные |
|---|---|
| Консоль/память | `--console series-s` (только CLI; `VICTORY_CONSOLE` не существует). Series S: title-memory 0 + **7776 MiB** (Series X: 10 GiB GPU-optimal + 3.5 GiB standard ≈ 13.5 ГиБ). `VICTORY_CARD_BUFFERS_MIN_KB=64`, `VICTORY_LEVELS_ON_DEMAND=1`, `VICTORY_UPLOAD_AHEAD=1` |
| Свет | `VICTORY_AMBIENT=0.8`, `VICTORY_AMBIENT_AIR=1.5` |
| Скорость | `VICTORY_THIN=1`, `VICTORY_CARD_STEPS=2`, `VICTORY_RELAY=2`, `VICTORY_RELAY_COPY=1`, `VICTORY_JOIN_PARTS=8`, `VICTORY_JOIN_COPIES=1`, `VICTORY_INDIRECT_BATCH=2`, `VICTORY_INDIRECT_WINDOW=1`, `VICTORY_MAP_THREADS=1`, `VICTORY_CARD_AWAKE=1`, `VICTORY_SHOW_AFTER=1`, `VICTORY_WIDE_BACK=1`, `VICTORY_CARD_REHASH_RUNS=1`, `VICTORY_TEXTURE_NO_WAIT=1` |
| Кэш/логи | `VICTORY_CACHE_DIR=%LOCALAPPDATA%\VIctory\cache-pareto`, `VICTORY_LOG=warn` |

Под 8 ГБ VRAM отдельного флага нет: форк **сам** режет лимиты карточных копий по DXGI Budget
(≤12 ГиБ → min 16 MiB / max 512 MiB; 16 ГиБ → 4/2048; ≥20 ГиБ → 1/мин(budget/4, 8192) МиБ), плюс
резиденция под 32 ГБ ОЗУ (давление 3 ГиБ RAM / 4 ГиБ commit, critical <1536 МиБ commit, staging
16/32 МиБ за кадр, prefetch off <4 ГиБ commit). У **нашего** дерева `card_limits` был фиксирован
`1 MiB / 1024 MiB` — на 8-ГБ карте это вдвое больше копий, чем у форка.

`VICTORY_AMBIENT` — **не «включение света»**, а стенд-ин непрямого света: после
`CS_ScreenProbes_TemporalResolve` подменяет трассированный indirect light оценкой из
`CS_CacheScreenSpaceRadiance` (64×36 ячеек → 160×90 направленных точек, складки по глубине,
ограничение рассеяния воздуха). Работает только когда лучи выключены (`builder_runs()` = false, т.е.
`VICTORY_BVH_BUILDER` не задан — так и есть по умолчанию). Значения: unset/`0`/`off` — выкл; число —
доля; `constant:<level>` — константа (вариант Belobug). Побочно лечит «brown veil / milky haze» воздуха.

### 19.3. Путь А против Пути Б — выбран **А**

| | Путь А: ветка от `coolfox/main` | Путь Б: перенос их коммитов к нам |
|---|---|---|
| Что получаем | весь их протестированный host-перф (THIN, LAND, CARD_STEPS, UPLOAD_AHEAD, JOIN_COPIES, AVX2-сравнение, descriptor records, HEAP_SPREAD), ambient, Series S, NVIDIA Wave32 lowering, 8-ГБ бюджеты, residency под 32 ГБ | наши шейдерные фиксы (#2268, NVIDIA #1176/#1249A) + наш perf-батч остаются |
| Риск/стоимость | отдельная ветка, полный перегрев кэша (~5 мин), **наши шейдерные фиксы на этой ветке отсутствуют** | **9 из 9 `git apply --check` = FAIL**: их модули `land.rs`/`safecopy.rs`/`card_steps.rs`/`thin.rs`/`queue/behind.rs`/`texture/ahead.rs` (556–912 строк каждый) у нас отсутствуют, `host.rs` разошёлся +4069/−2951, `descriptor.rs`/`shadow.rs` переписаны; это не порт, а переписывание |
| Верифицируемость | автор собирал и гонял это дерево; `cargo check --locked` и релиз у нас прошли | каждый кусок пришлось бы чинить вручную без эталона |

Наша ветка `visual-and-nvidia-fixes` при этом **не тронута** — она остаётся в `myfork` как есть.

### 19.4. Что сделано и с какими числами

- `git remote add coolfox https://github.com/codec-d/cool_fox.git` + `git fetch coolfox` (2,1 с).
- Ветка **`coolfox-rtx`** от `coolfox/main` (`f4b48ad`); добавлены `AGENTS.md`, `HANDOFF.md` (наши, из
  `visual-and-nvidia-fixes`) и `victory.toml` (`game_dir=C:\GTA6_Files`, `cache_dir=…\VIctory\cache-pareto`).
  `victory.exe config` → exit 0, пути приняты.
- `cargo check --workspace --locked` → **exit 0** (12 предупреждений в `victory-gpu` — их собственные).
- `cargo build --release --locked -p victory` → **exit 0, 1 м 05 с**; `victory.exe` **9 979 904 Б**, 13:13:18.
- Прогрев кэша под их транслятор (отдельная папка, чтобы не смешивать с нашими 25 637):
  `all hashes asked, 25889 found: 0 in the cache already, **25 663 translated now**, 226 do not translate; 294,3 с`;
  в `cache-pareto` 25 663 файла / 892 МБ.
- `translate --list`: **226 непереведённых** — `195 resources: an address in an SGPR that is no root parameter`
  (`GS_VoxelizeGeneric`; у нас это лечит #2268), `26 emit: no wave32 lowering for this wave64 program`,
  `5 emit: a descriptor read only as data` (`CS_TraceShadowRays`). Наша ветка: 57 (40 descriptor-phi +
  10 SGPR + 5 shadow + 2 маски) — **т.е. их транслятор лучше на descriptor-phi/SGPR (~50 программ), но не
  содержит нашего #2268 и нашего NVIDIA wave-wide слоя**.

### 19.5. Команды (PowerShell, из `C:\GTA6_Port\victory-fork`)

Прогрев уже сделан; повторять только после правок `victory-shader`:
```powershell
$env:VICTORY_HOST_WAVE='32'; $env:VICTORY_VIEW_LIMIT='999999'; $env:VICTORY_CACHE_DIR="$env:LOCALAPPDATA\VIctory\cache-pareto"; .\target\release\victory.exe translate 'C:\GTA6_Files\shaders.rpf' --warm all --jobs 12
```

**Запуск игры (основной вариант — авторский пресет через их лаунчер):**
```powershell
$env:VICTORY_GAME_DIR='C:\GTA6_Files'; $env:VICTORY_DISPLAY_MODE='1920x1080'; $env:VICTORY_LOG='warn'; .\Play-VIctory.cmd pareto-residency
```

Тот же запуск без `.cmd` (если захотите менять флаги по одному):
```powershell
$env:VICTORY_GAME_DIR='C:\GTA6_Files'; $env:VICTORY_CACHE_DIR="$env:LOCALAPPDATA\VIctory\cache-pareto"; $env:VICTORY_AMBIENT_AIR='1.5'; $env:VICTORY_TEXTURE_NO_WAIT='1'; $env:VICTORY_RELAY='2'; $env:VICTORY_WIDE_BACK='1'; $env:VICTORY_CARD_REHASH_RUNS='1'; $env:VICTORY_JOIN_COPIES='1'; $env:VICTORY_INDIRECT_WINDOW='1'; $env:VICTORY_RELAY_COPY='1'; $env:VICTORY_CARD_STEPS='2'; $env:VICTORY_INDIRECT_BATCH='2'; $env:VICTORY_MAP_THREADS='1'; $env:VICTORY_CARD_AWAKE='1'; $env:VICTORY_SHOW_AFTER='1'; $env:VICTORY_AMBIENT='0.8'; $env:VICTORY_THIN='1'; $env:VICTORY_JOIN_PARTS='8'; $env:VICTORY_UPLOAD_AHEAD='1'; $env:VICTORY_LEVELS_ON_DEMAND='1'; $env:VICTORY_CARD_BUFFERS_MIN_KB='64'; $env:VICTORY_DISPLAY_MODE='1920x1080'; $env:VICTORY_LOG='warn'; .\target\release\victory.exe run 'C:\GTA6_Files\game_scarlett_master.exe' --console series-s --execute
```

Полезные подмены: `--console series-x` вместо `series-s` (больше title-памяти, ближе к их замерам, но
хуже для 8 ГБ); `$env:VICTORY_AMBIENT='0'` — выключить стенд-ин света; `$env:VICTORY_AMBIENT='constant:0.6'`
— константа Belobug; `$env:VICTORY_LOG='info'` — увидеть `host wave lanes`/`host view heaps` (нужно,
если когда-нибудь пересобирать кэш); `-RayTracing` в `play.ps1` включает `VICTORY_BVH_BUILDER=1`
(**не для 8 ГБ**: builder заявлен как «+40 мс/кадр», исторически писал мусор в title-память и ронял игру).

### 19.6. Риски и что дальше

1. **Ambient — хак, не GI**: позиции в root-таблицах зашиты (`LIGHT=(1,176)`, `DEPTH=(0,464)`,
   `RADIANCE=(1,16)`, `AIR=(1,32)`); если наша упаковка таблиц отличается — попадёт в чужой слот
   (симптомы: пересвет/артефакты). Откат без пересборки: `VICTORY_AMBIENT='0'`.
2. **Series S** режет title-память до 7776 МиБ — на 8 ГБ VRAM это плюс, но игра может чаще стримить
   (на слабом CPU это кадры). Переключение — только `--console series-x`.
3. **Теней/RT по-прежнему нет**: 5 `CS_TraceShadowRays` не переводятся ни у них, ни у нас; builder off.
   Т.е. «свет в помещениях» после пресета — это ambient-стенд-ин, а не настоящий GI.
4. **Наши наработки на этой ветке отсутствуют**: нет #2268 (195 GS), нет NVIDIA #1176/#1249A (волновые
   формы). Логичное продолжение — отдельной задачей перенести в `coolfox-rtx` наш шейдерный слой
   (#2268 + wave-wide) **одним батчем**, затем один перегрев кэша (~5 мин). Это вернёт 195 GS в tracking
   и, вероятно, переведёт часть 26 wave32.
5. 226 непереведённых можно уменьшить до ~5–31, объединив обе линии; сейчас на этой ветке их 226.
6. Все замеры авторов — AMD RDNA3 / RTX 5090 на 32 ГБ; на 3060 Ti 8 ГБ поведение не проверено ни ими,
   ни нами. `VICTORY_RDNA_FLOAT` и их wave-формы AMD-ориентированы — не смешивать с нашим NVIDIA-слоем.

### 19.7. Файлы

- Отчёты субагентов: `C:\GTA6_Port\coolfox\launcher\` (`REPORT.md`, `preset-pareto-residency.md`,
  `comparison-our-tree.md`, `security-audit.md`, `sources\Play-VIctory.cmd.txt`, `sources\play.ps1.txt`,
  `fork-vars-unique.txt` — 345 имён `VICTORY_*`), `C:\GTA6_Port\coolfox\perf\` (`SUMMARY.md`, `patches\`
  01-thin…09-upload-ahead, `apply-check-results.txt`, доки форка), `C:\GTA6_Port\coolfox\light\`
  (`REPORT.md`, `TRANSLATOR-DIFF.md`, `ambient.rs`, версии `lower.rs`/`emit`/`resource.rs`/`native.rs`);
- Сборка/прогрев этой ветки: `C:\GTA6_Port\coolfox\check-coolfox.log`, `build-coolfox.log`,
  `warm-coolfox.log`, `translate-list-coolfox.txt`;
- Ветка `coolfox-rtx` в `myfork`; наша `visual-and-nvidia-fixes` @ `4133084` — без изменений.

## §20. Тестовый запуск coolfox-rtx на RTX 3060 Ti: замер и сравнение с нашей веткой

Задание: прогнать coolfox-rtx (`7493bc8`) авторским пресетом `pareto-residency` в 1920x1080, реальными
нажатиями (`sendkeys2.ps1`) пройти меню до 3D-мира и сцены у окна/допроса, снять FPS/VRAM/скриншоты,
проверить ambient и пропажи эффектов, записать сравнение с `visual-and-nvidia-fixes` и запушить HANDOFF.
Код не менялся — только флаги/CLI.

### 20.1. Что запускалось (два прогона, оба 1920x1080, RTX 3060 Ti 8 ГБ, 32 ГБ ОЗУ)

1. **Прогон 1 — авторский пресет** `.\Play-VIctory.cmd pareto-residency` (он жёстко шлёт
   `--console series-s`), `VICTORY_GAME_DIR=C:\GTA6_Files`, `VICTORY_DISPLAY_MODE=1920x1080`,
   `VICTORY_LOG=info`; 340 с; подпись окна (fps/ms/draws) каждую секунду, `nvidia-smi` каждые 2 с,
   скриншот окна каждые 30 с; нажатия по проверенной схеме (`Enter` на 15/30 с, адаптивные `Enter`
   каждые 10 с пока в меню, один `Down+Enter` на EULA, `Enter` в мире на 165/205/245/285/320 с).
   Логи: `C:\GTA6_Port\run-logs-coolfox\` (out/err/captions/vram/keys, 11 скриншотов).
2. **Прогон 2 — диагностический (те же флаги, другой только `--console series-x`)**, 320 с, то же
   логирование, скриншоты каждые 20 с с меткой фокуса. Логи: `C:\GTA6_Port\run-logs-coolfox2\`
   (15 скриншотов, все валидные).
Оркестраторы: `C:\GTA6_Port\run-coolfox.ps1`, `C:\GTA6_Port\run-coolfox2.ps1`; нажатия — проверенный
`C:\GTA6_Port\sendkeys2.ps1` (фокус через Alt-разблокировку + AttachThreadInput, затем `keybd_event`).

### 20.2. FPS и VRAM (мир: от первого кадра с 500+ draw)

| | coolfox-rtx, **серия S** (`.cmd`) | coolfox-rtx, **серия X** | наша `visual-and-nvidia-fixes` r6 | наша r7 |
|---|---|---|---|---|
| Мир начался | 139 с | **71 с** | 146 с | 141 с |
| Выборок подписи | 163 | 243 | 194 | 199 |
| **FPS медиана** | **3** | **2** | **6** | **6** |
| FPS среднее | 2,88 | 1,90 | 5,27 | 4,55 |
| FPS min / max | 0 / 4 | 0 / 3 | 0 / 9 | 0 / 8 |
| мс/кадр медиана | 306 | 536 | 163,8 | 181,7 |
| draw/кадр медиана | 3628 | 5031 | 3946 | 3747 |
| **Пик VRAM (nvidia-smi)** | **7977 / 8192 МиБ (97 %)** | 7915 МиБ | 7900 МиБ | 7919 МиБ |
| VRAM медиана | 7554 МиБ | 7777 МиБ | — | — |
| Разложение кадра (сводка лога) | preparing 98 мс (копии 26–38 мс/95–102 МБ, текстуры 0,5–9 мс), **GPU 48–66 мс**, назад 70–81 мс/47 МБ | preparing 95 мс, **GPU 241 мс**, назад 104 мс/42 МБ | preparing 95 мс (копии 51,7 мс/99,5 МБ, текстуры 7,6 мс), **GPU 5,0 мс**, назад 76 мс/67,5 МБ | — |
| Брошенные draw | 300–310/кадр (**300 ещё создавались**) | 313/кадр (0 создавались) | 369/кадр (0 создавались) | — |
| Текстуры по индексу | 13 013 залито (4,3 ГБ), **25 905 ждут**, 5 595 без маппинга | 14 267 (4,0 ГБ), **28 307 ждут**, 4 270 | 44 730 (17,2 ГБ), 14 701 ждут, 13 446 | — |

Вывод по скорости: **на 3060 Ti coolfox-rtx в 2–3 раза медленнее нашей ветки в той же сцене**.
Причина видна в разложении: у нас карта простаивала (GPU 5 мс, кадр упирался в подготовку и
копирование назад), у coolfox карта стала узким местом (GPU 48–241 мс), при этом «подготовка»
и обратное копирование остались такими же, а очередь текстур стала хуже (28k ждут против 14,7k).
Их механизмы (`THIN`, `CARD_STEPS=2`, `RELAY_COPY`, `UPLOAD_AHEAD`, `SHOW_AFTER`, ambient-проходы)
на этом железе добавляют работы карте и не снимают наш главный тормоз — копирование назад.

### 20.3. Главная находка: Series S = чёрный экран

- Лог прогона 1: `display output: compositing two planes (overlay active) frame=1 width=1920 height=1080
  base_format=28 overlay_format=28` — в режиме Series S включается **two-plane композит**
  («base + overlay»). При этом 3000–3600 draw/кадр честно выполняется, подпись окна показывает
  3 fps/306 мс, а **на экране чёрный кадр**: на скриншотах 122–248 с видны только субтитры
  («ПРОЛОГ») и краевые элементы UI, 3D-картинки нет. Окно при этом 1354×818 (динамическое
  разрешение под Series S), у нас — 1920×1080.
- Прогон 2 с `--console series-x` (те же флаги, кроме модели консоли) — **картинка появилась**
  с первого же кадра мира: окно 1920×1080, сцена у окна (тюремный двор, закат, люди), затем
  коридор с лампами. Все 15 скриншотов валидны.
- Вывод: на нашем дампе (`C:\GTA6_Files`) путь two-plane Series S из cool_fox не работает — их
  коммит `1ad6e67` («Fix Series S two-plane black screen by reading RetailPlane at 72-byte stride»)
  заточен под их раскладку; наш дамп даёт чёрный overlay. **Запускать coolfox-rtx надо с
  `--console series-x`, а не через `Play-VIctory.cmd`** (он жёстко шлёт `series-s`).

### 20.4. Графика: ambient, пересвет, пропажи

- **Ambient работает**: `victory_gpu::ambient` пишет 4 строки (роли `Radiance` 1280×720/640×360,
  `Resolve`, `Air` 160×90, level 0.8) и в прогоне 2 включает подмену сразу на входе в мир.
  На кадре 303 с (интерьер коридора с лампами) **глухой черноты нет**: стены и спица персонажа
  освещены, коридор читается; на кадре 263 с — профиль у окна, интерьер тёмный, но с деталями.
  Против нашей ветки (кадр 241 с из `run-logs7`: силуэт почти в полной черноте) разница заметна.
- **Пересвет**: жёсткого выжигания нет, но есть (а) «молочная дымка» в кадрах перехода/заката
  (кадр 143 с) — это ровно тот `brown veil/milky haze`, который автор лечит `VICTORY_AMBIENT_AIR=1.5`;
  (б) локальный пересвет спины персонажа в коридоре (кадр 303 с) — ambient добавляет света
  без теней, поэтому выход за 1.0 возможен. Не критично, но яркость стоит убавить
  (`VICTORY_AMBIENT=0.6`) при следующем прогоне.
- **Пропажи из-за шейдеров**: за прогон дропнуто 3 compute-программы с
  `no wave32 lowering for this wave64 program`: `02384a60fda6f2d0`, **`5dc3b25986622949`** и
  **`6f225b641c5b0708`**. Последние два — как раз водные волюметрики, которые **наша ветка
  переводит** благодаря NVIDIA-слою #1176/#1249A (в наших прогонах дропаются только
  `2da4beb3`/`02384a60`). Т.е. на coolfox-rtx эффекты воды/волюметрики отсутствуют там, где
  они есть у нас. Также в прогоне 1 был `refused dll="amd_fidelityfx_upscaler_xbox.dll"`
  (FSR-апскейлер вне host-policy — в обоих деревьях), а в прогоне 2 — 1 событие
  `the card has no room for more` (VRAM 97 %).
- **Текстуры/геометрия**: в проверенных кадрах пропавших текстур и дыр в геометрии не видно —
  персонажи, двор, коридор с лампами на месте. Но очередь текстур хуже нашей (25,9–28,3 тыс. ждут
  против 14,7 тыс.), и `behind`-механики не успевают: часть текстур показывает неполный mip.
  Теней/RT по-прежнему нет (builder off, 5 `CS_TraceShadowRays` не переводятся ни у них, ни у нас).

### 20.5. Итог и рекомендация

| Критерий | coolfox-rtx | visual-and-nvidia-fixes |
|---|---|---|
| Картинка в мире | **только `--console series-x`**; `.cmd`-пресет = чёрный экран | рисует сразу (series-x по умолчанию) |
| FPS в той же сцене | медиана **2–3** | медиана **6** |
| GPU-время | 48–241 мс (карта — узкое место) | 5 мс (карта простаивает, кадр упирается в копирование) |
| Свет в интерьере | **ambient-стенд-ин включён**: не чёрно | интерьер тёмный (ambient нет) |
| Шейдеры | 226 не переводятся; **2 водные дропаются** | 57 не переводятся; водные переведены |
| Кэш шейдеров | отдельная папка `cache-pareto`, 25 663 | 25 637, ключ свой |

**Рекомендация:** основной линией остаётся `visual-and-nvidia-fixes` (быстрее в ~2–3 раза и без
чёрного экрана); coolfox-rtx держать как экспериментальную ветку для света и запускать только
`--console series-x`. Правильный следующий шаг — **перенести `victory-gpu/src/ambient.rs` из
coolfox-rtx в нашу ветку** (это `gpu`-файл, кэш шейдеров не сбрасывает; ~530 строк + 3 флага
`VICTORY_AMBIENT`/`_AIR`/`_SHADE`), а не наоборот: переносить весь cool_fox в нашу линию нечем
(9/9 `apply-check` = FAIL), а наша линия уже быстрее и с рабочим Series X.

### 20.6. Штатное закрытие не сработало

`CloseMainWindow()` вернул `True` в обоих прогонах, но процесс **не завершился**: ждали 60 с
(прогон 1) и 45 с (прогон 2), после чего пришлось убивать. Кэш шейдеров от этого не пострадал
(25 663 файла на месте), но новые пайплайны сессии в `cache-pareto` почти не попали: в прогоне 1
кэш вырос с 25 663 до **25 695 (+32 файла)** — вероятно, это записи образов/частичные, а не
полный pipeline book. Т.е. «новые пайплайны записались» выполнилось лишь частично — нужен
штатный выход через меню игры, а не WM_CLOSE (у тайтла свой обработчик закрытия).

### 20.7. Файлы

- Логи прогона 1 (Series S): `C:\GTA6_Port\run-logs-coolfox\` — `captions.log`,
  `vram.log`, `keys.log`, `watch.log`, `out.log`, `err.log` (15 828 строк), `shot-*.png` (11),
  `cache-before.txt`/`cache-after.txt`;
- Логи прогона 2 (Series X): `C:\GTA6_Port\run-logs-coolfox2\` — то же + 15 валидных
  `shot-*_f0.png`; лучшие кадры: `shot-203_f0.png` (окно/двор), `shot-263_f0.png` (профиль у окна),
  `shot-303_f0.png` (коридор с лампами, ambient);
- Оркестраторы: `C:\GTA6_Port\run-coolfox.ps1` (авторский `.cmd`), `C:\GTA6_Port\run-coolfox2.ps1`
  (Series X; известный дефект: параметр функции `$pid` конфликтует с read-only `$PID`, поэтому
  метка фокуса всегда `_f0`, но нажатия работают через `sendkeys2.ps1`);
- Для сравнения: `C:\GTA6_Port\run-logs6\` (наша ветка, prefetch=0) и `C:\GTA6_Port\run-logs7\`
  (prefetch=1).

## §21. Ветка `road-to-20fps`: ambient-свет, предсборка пайплайнов и контрольные замеры (2026-11-26)

### 21.1. Что сделано (ветка `road-to-20fps` от `visual-and-nvidia-fixes` @ `4133084`)

| Шаг | Коммит | Содержание |
|---|---|---|
| 0 | `b4ebb15` | AGENTS.md п.7 (один логический шаг, A/B-контроль, проверка дескрипторов/root-таблиц и 8 ГиБ VRAM, откат при регрессе) + перенос §20 в эту ветку |
| A | `450526e` | `victory-gpu/src/ambient.rs` из `coolfox-rtx` (530 строк) + интеграция: `Shader.ambient`, `State.ambient_tables`, capture в `bind`, dispatch-хук, `fn ambient()`; флаги `VICTORY_AMBIENT` / `_AIR` / `_SHADE`; `victory-shader` не тронут, кэш шейдеров цел |
| — | `3f03c23` | `VICTORY_KEY_PAD=1` (gameinput.rs): кнопки клавиатуры добавляются в подключённый пад 0 — без этого автопрогоны стояли на экране выбора учётной записи (XInput-слот 0 на машине занят) |
| B | `5163221` | PR #1226 `fxceless/victory-fork` (regiumlepidi, head `7ceeed09`, base `f6a09d3`): `translate --cache [--pipelines]`, `victory-game::{cityhash,shaderpack,pipelines,sgd5}`, `pipeline::{read_stream,Prebuild,prebuilt,wanted_early}`, `make::key`, `library::start_prebuild`; `victory-shader` не тронут |

Проверки каждого шага: `cargo check --workspace` — exit 0; `cargo test --workspace` — **607 passed, 0 failed** (Step B); `cargo build --release -p victory --locked` — exit 0 (36,7 с инкрементально).

### 21.2. Аудиты (3 параллельных read-only разбора, `C:\GTA6_Port\audit\`)

1. **`ambient.md` (42 КБ, вердикт GO)** — построчная сверка переноса из `coolfox-rtx` @ `281ad6b`: host-only стенд-ин, дескрипторы root-таблиц LIGHT=(1,176), DEPTH=(0,464), RADIANCE=(1,16), AIR=(1,32) — поля XDAT заголовка; конкурентных записей нет, VRAM не растёт ; `victory-shader` не затронут → 25 637 переводов остаются валидными.
2. **`cpu.md` (70 КБ + графики)** — что реально в кадре: «5 мс on the GPU» — это **ожидание CPU**, а не время карты (карта занята 73–76 %); кадр 1080p ≈ preparing 92,7 + descriptor sync 13,4 + copy-back 64,4 мс; 720p поднимает FPS на проценты (кадр упирается в байты/слоты/draws); провалы меню — бюджет запекания текстур `VICTORY_BAKE_TEXTURES_MS` (1500 мс, до 2 ГиБ/кадр) + стриминг тайтла + редкие PSO-компиляции; прогноз: M0+`ASYNC_PRESENT` ≈ 8–11 FPS, для 15–20 FPS нужен структурный срез ~100 мс (copy-back ring, параллельность heap, descriptor sync).
3. **`pr1226.md` (333 строки, GO + готовый патч `pr1226-port.diff` 118 316 Б)** — предсборка: сухой прогон на нашей машине 28 164 пайплайна (27 160 ключей, 1 004 пропуска), пересечение с библиотекой 2 315 → к сборке ≈24 600; оценка 13–20 мин (факт — 28,3 мин, см. 21.4); кэш шейдеров и формула ключей пайплайнов не изменены.

Важно: логи `run-logs6/7` и старые замеры — **до** perf-батча, для текущей ветки не действительны.

### 21.3. Шаг A: ambient (свет в интерьере), замеры M1 и M0b

Флаги прогона: `VICTORY_AMBIENT=0.6`, `VICTORY_AMBIENT_AIR=1.5`. В M1 лог подтвердил 4 роли: `Radiance 1280x720`, `Resolve 2560x1440`, `Air 160x90` (×2) — стенд-ин работает. VRAM-пик не вырос: M1 **7839/8192 МиБ** против M0 (7888) и M0b (7901).

Картинка (кадры `road\m0b\shot-333_f1.png` без ambient vs `road\m1\shot-321_f1.png` с ambient): та же спальня — у нас стены/потолок/койки читаются, персонажи освещены отражённым светом; без ambient стены почти чёрные. Явного пересвета нет (средняя яркость 46–51 у обоих; «washed»-метки — только на светлых переходных кадрах/заставках, как и без ambient).

**Вердикт A/B (M0b ambient-off — тёплый контроль — против M1 ambient-on):**

| Показатель (сводки, draws ≥ 500) | M0b (off) | M1 (on) |
|---|---|---|
| кадров | 58 | 51 |
| fps медиана / среднее | 4,1 / 3,69 | 3,9 / 3,59 |
| frame ms медиана | 250,1 | 256,7 |
| prepares медиана | 149,1 | 156,0 |
| GPU-ожидание | 45,3 | 39,7 |
| copy-back ms медиана | 100,3 | 124,3 |
| dropped медиана | 388 | 347 |

По корзинам draws медианы fps: 4500–5500 — 4,3 vs 4,7; 5500–7000 — 3,7 vs 3,3; 7000+ — 4,4 vs 4,6; 3500–4500 — 5,3 vs 3,9; 1500–2500 — 4,8 vs 2,9 (малые n, шум). Систематического падения >10 % нет: разница в пределах разброса повторных прогонов (см. 21.5, M2 4,0 против M2b 4,9). **Шаг A оставлен** (визуальный выигрыш реальный, стоимость в шуме). Откат при регрессе: `git revert 450526e` (или просто не задавать `VICTORY_AMBIENT`).

### 21.4. Шаг B: предсборка пайплайнов (PR #1226) — принята

Pre-flight A (`translate --cache`, без `--pipelines`): `containers 25985 in 5.5 s: 25698 already cached, 0 translated, 287 refused` — env/сборка совпали, кэш цел. Первый заход (до обрыва) дал 58 translated — новые контейнеры из exe/DLL, ожидаемо.

Pre-flight B (`VICTORY_PREBUILD_KEYS`): `28164 in the pack: keys of 27160`; пропуски ровно как в аудите: 3 blend + 139 depth-stencil + 852 rasteriser + 8 shader not found + 2 stages = 1 004; в кэш записано 0 файлов.

Полная предсборка (`translate --cache --pipelines --jobs 12`, 4 файла; лог `road\prebuild\prebuild.log`):
`pipelines 28164 in the pack, 1699.3 s on 12 threads: 21505 made, 5641 already in the library`.
Итоговая библиотека `host-pipelines.bin` — **1 090 771 490 Б** (было 282 992 100), `prebuilt-pipelines.txt` — 888 094 Б; свободно 118,7 ГиБ (было 122,5).

Повторный запуск (критерий приёмки): `28164 in the pack, 5.8 s: 0 made, 27146 already in the library` — в кэш/библиотеку не записано ни одного нового пайплайна, кроме 22 «догруженных» из уже собранной библиотеки.

Кэш шейдеров **не изменён**: до и после предсборки — **76 973 файла / 2 614 092 344 Б** (те же 25 637 переводов). Ключи пайплайнов те же (формула не менялась). Заметно, что реальное время вышло ~28 мин против оценки 13–20: первые ~1 000 пайплайнов собирались ~6,5 мин (тяжёлые шейдеры + загрузка), дальше темп вырос до ~19/с; итого 12,6 пайплайна/с против 45/с у автора (у него 9800X3D; наша 10600 + 3060 Ti). Библиотека вышла 1,09 ГБ, а не 2,6 — записи в среднем ~40 КБ.

Приёмка B выполнена: check/test зелёные; предсборка нашла 28 164 и собрала 21 505; второй запуск — 0 made; кэш шейдеров не тронут; откат — `git revert 5163221` + вернуть библиотеку из `C:\GTA6_Port\pipelines-backup-before-prebuild\` (или оставить предсобранную из `pipelines-backup-prebuild\`).

### 21.5. Контрольные замеры (все — 1280×720, topmost, один и тот же бинарь/env)

Прогоны (логи `C:\GTA6_Port\road\`): **M0** — база (холодный кэш), **M0b** — тёплый контроль ambient-off, **M1** — ambient 0.6/1.5, **M2** — после предсборки, **M2b** — повторный контроль async-off, **M3** — `VICTORY_ASYNC_PRESENT=1`.

Сводки по кадрам мира (draws ≥ 500; preparing — работа готовящего потока):

| Прогон | n | fps мед. | fps сред. | ms мед. | preparing | copies | textures | GPU-ожид. | copy-back | draws | dropped | VRAM пик |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| M0 | 33 | 4,4 | 4,34 | 227 | 117,3 | 61,8 мс | 12,0 | 47,6 | 80,6 | 3187 | 357 | 7888 |
| M0b | 58 | 4,1 | 3,69 | 250 | 149,1 | 88,2 мс | 11,3 | 45,3 | 100,3 | 5284 | 388 | 7901 |
| M1 | 51 | 3,9 | 3,59 | 257 | 156,0 | 100,4 мс | 11,7 | 39,7 | 124,3 | 4833 | 347 | 7839 |
| M2 | 59 | 4,0 | 3,63 | 252 | 159,1 | 100,1 мс | 13,0 | 44,0 | 121,3 | 5037 | 358 | 7848 |
| M2b | 58 | 4,9 | 4,31 | 205 | 155,3 | 91,2 мс | 11,7 | 32,7 | 127,5 | 4873 | 367 | 7665 |
| M3 | 70 | 4,6 | 4,17 | 219 | 136,9 | 92,9 мс | 8,5 | 54,9 | 108,4 | 4790 | 362 | 7954 |

По секундным подписям окна (captions): M0 5,0/185 мс (109 кадров мира), M0b 4,0/232 (222), M1 4,0/233 (197), M2 4,0/235 (237), M2b 5,0/202 (242), M3 5,0/209 (263). Разброс одинаковых прогонов M2/M2b (4,0 против 4,9) больше, чем разница между любыми двумя шагами, — вывод: одиночные прогоны для A/B флагов недостаточны, нужны 2–3 на плечо и сравнение по корзинам draws.

Меню: making-max 2 (M0), 88 (M0b), 1421 (M1), 6 (M2), 1457 (M2b), 1251 (M3). Т.е. очередь «being made» после предсборки — это **догрузка из библиотеки** (компиляций нет: `made` в прогоне 0), а не PSO-компиляция: падение FPS в меню, которое лечил Step B, приходило именно из компиляций; остальные провалы меню (p99 200–1800 мс, stalls 0–3) — запекание текстур/стриминг, это следующий фронт.

**Вердикт по предсборке:** средний FPS мира не изменился (4,0 против 3,9–4,1 у M1/M0b), dropped не изменился (~350–390 — это постоянные отказы, а не «в процессе»), но прогоны после предсборки **не компилируют пайплайны вообще** (`0 made`, библиотека не переписывается) — исчезает источник длинных hitch'ей первого знакомства со сценой и стартовых штормов (M0 cold: меню p99 5178 мс, M1: making 1421). Плюс это страховка для 8 ГиБ VRAM: компиляции не конкурируют с кадром.

### 21.6. M3: `VICTORY_ASYNC_PRESENT=1` (A/B без кода)

M3 (async on): 4,6 fps / 219 мс медиана; M2 (off): 4,0/252; M2b (off, повтор): 4,9/205. По корзинам draws 3500–4500 — M3 5,4 против 4,1/2,3 у M2/M2b; 4500–5500 — 4,5 против 4,5/5,1; 5500–7000 — 4,7 (у M2/M2b мало данных). GPU util в M3 64 % против 43 % у M2/M2b (async present не даёт потоку ждать), VRAM 7954 (все ещё без «no room for more»). **Вывод: эффект в пределах разброса прогонов; включать по умолчанию пока не следует** — как опция флаг оставлен, для решения нужны повторные пары.

### 21.7. Где сейчас кадр (по M2b/M3) и что это значит

- preparing 137–159 мс: copies 91–100 мс (110–136 МБ/кадр), textures 8–13 мс; copy-back 108–128 мс (67–78 МБ); GPU-ожидание 33–55 мс (карта не узкое место, util 38–64 %).
- 4 800–5 300 draws/кадр, ~360 draws дропаются постоянно (отказы трансляции — 1 018 пропусков пака + 14 отказов хоста; предсборкой не лечатся).
- VRAM 7,7–7,9 ГиБ из 8 — потолок близко; сцены без «room for more» держатся на TEXTURE_MAX_DIM=1024/94 %.
- Следующий структурный срез — copy-back ring/меньше байт туда-обратно, параллельность heap-пасса и descriptor sync (это ровно то, что делают свежие коммиты fxceless, см. 21.8).

### 21.8. Разведка свежих коммитов `fxceless/victory-fork` («hyphennn») для следующей итерации

Форк: `fxceless/victory-fork` (бывш. `victory-gta6`, 3 190★), `main` HEAD на 2026-11-26 11:18Z — `ceb9e1f7`; README обновлён 11:50Z: **8,2–8,6 fps** на первой лодочной миссии, 9,5 на стоячем прогоне, 10–14 на лёгких видах (их машина: i7-14700KF + RTX 4070 12 ГБ); с `VICTORY_HEAP_WORKER=1` было 6,2 → 8,65–9,53. Потеря карты через 30–50 мин (0x887A0005) — открытая проблема и у них; тени вне прямого солнца у них всё ещё чёрные (наш ambient — то, чего у них нет). Свежие «скриншоты hyphennn» — это встроенный в кадр FPS-счётчик (`fpsbox.rs`, `VICTORY_FPS_COUNTER`, у нас есть) и «corner box» с загрузкой CPU/самого горячего потока/GPU/VRAM/RAM (коммит `2095f154`, `stats.rs` — **у нас нет**).

Проверено grep'ом по нашему дереву: перечисленные ниже файлы/переключатели у нас отсутствуют. Приоритет на следующую итерацию:

1. **`d4a528ba` + `f6a09d3b` — пасс текстур на своём потоке (`VICTORY_HEAP_WORKER=1`, PR #1007)**: у них +40 % кадров мира на лодке (6,2→8,65…9,53) — самый крупный оставшийся шаг. Новый файл `crates/victory-gpu/src/heap_worker.rs` (~542 строки) + правки `descriptor/host.rs`, `draw.rs`, `host.rs`, `lib.rs`, `list.rs`, `object.rs`, `resource.rs`, `texture.rs`; `f6a09d3b` — замер, `VICTORY_REAPER`, `VICTORY_BAKE_SOURCES` (PR #1264), `VICTORY_TILES_SMALL_WHEN_FULL_MB`, `VICTORY_RETIRE_LEAN`.
2. **`06702fe8` — `VICTORY_SETTLE_ROOM=1`**: −11,5 мс готовящего потока/кадр при равных draws (ожидание карты 17,7→4,6 мс); compute-диспетчи >65 535 групп разбрасываются по z (upstream PR #2312, терялось 43 % пасса); BC7-ротация только в режимах 4–5 (upstream PR #3111); `VICTORY_REASK_BLANK`, `VICTORY_DENY_PIPELINES`, инструменты DirectStorage. 19 файлов, +1456/−64.
3. **`ceb9e1f7` — `VICTORY_PARALLEL_REFRESH=1`**: −8,7/−9,4 мс готовящего потока (проверки текстур параллельно на helper-потоках, ×3,6–3,8), с `SETTLE_ROOM` суммарно ~−20 мс; самые медленные запросы DirectStorage; переименование `VICTORY_DISPATCH_SPREAD`. 6 файлов, +270/−6.
4. Мелкие opt-in того же дня: `5d659e0c` (`VICTORY_TABLE_STAGES`, `VICTORY_HEAP_LOCKS`), `895a23bc` (`VICTORY_HEAP_PASS_MARKED`, `VICTORY_LIST_REUSE`), `94ed363c` (`VICTORY_LIGHT_REFRESH`, `VICTORY_ABSENT_*`), `84c5dc00` (`VICTORY_STEP_INDEX`), `de4146a0` (`VICTORY_FILLS_PLAIN`), `006c3530` (`VICTORY_GATHER_BY_COPY`), `4679615a` (`VICTORY_VIEWS_REMADE_QUIET`), `4d0c6345` (свободный прогон dynamic-слотов), `6d20eed6` (`VICTORY_CARD_HASH_WIDE` — у них замерено «без выигрыша»).
5. Корректность/стабильность (важно для 8 ГиБ): `025b042a` — целочисленные render target'ы не смешиваются (фикс из debug layer; в аудите PR #1226 отмечен как единственная base-фича, которой нет в нашем `pipeline.rs`); `a35c5573` — `VICTORY_TARGET_VARIANTS` (пайплайн под формат связанного RTV); `2ba6e1f9` + `a183ea27` — CBV ограничены 64 КиБ (PR #911; «невалидный вызов → device removal» на чужой карте — наш риск); `be721d5a`/`619b2dc5` — CBV нулевого размера остаётся одним (PR #1260, мигающий пол); `19a45d19` — PR #1241 (вью текстуры, записанной картой).
6. Шейдеры (осторожно: у нас свои #2268/#1176, кэш 25 637): `96ff1356` — `VICTORY_BVH_NO_HIT` + `VICTORY_WAVE_WIDE=last` (PR #1141); `c36971b2` — `VICTORY_SHADER_OPT` бит 64 (скалярные загрузки по 4 dword'а). Любой шейдерный перенос — только со сверкой индексов и перепрогревом кэша.
7. Диагностика для наших замеров: `2095f154` corner box CPU/GPU/VRAM/RAM (`stats.rs`), `0672327d` строка direct-очереди карты, `d8986cd7` разбивка времени ремапа, `b9011c92` память при потере устройства.

Открытые PR форка в тему (на 11:40Z): #1290 (цепочка heap-worker пассов и раннее чтение текстур — сверить с `d4a528ba`/`f6a09d3b`), #1292 (DispatchX), #1289/#1280 (fence-safe LRU текстур и headroom), #1274 (страницы по 2 МиБ), #1286 (decommit RAM), #1287 (loose-LDS fallback).

### 21.9. Итог и следующий шаг

Сделано и принято: ambient (интерьер больше не чёрный, FPS в шуме), предсборка пайплайнов (0 компиляций в прогоне, кэш шейдеров и ключи целы), инфраструктура автопрогонов (`VICTORY_KEY_PAD`), правило AGENTS п.7. Кадр мира по-прежнему ~205–260 мс: копии/copy-back и descriptor sync — главное; средний FPS мира не вырос, и это ожидаемо по аудиту (`cpu.md`).

Рекомендация следующей итерации (по одному шагу, каждый с A/B в 1280×720): (1) `VICTORY_HEAP_WORKER` (d4a528ba+f6a09d3b) с замером; (2) `SETTLE_ROOM`+`PARALLEL_REFRESH`; (3) стабильность/CBV 64 КиБ/integer RT/TARGET_VARIANTS; (4) мелкие opt-in switch'и. Для A/B флагов делать **2–3 прогона на плечо** (разброс M2/M2b = 0,9 fps) и сравнивать по корзинам draws.

### 21.10. Файлы

- Логи/кадры замеров: `C:\GTA6_Port\road\{m0,m0b,m1,m2,m2b,m3}\` (captions/vram/watch/err.log, shot-*.png, flags.txt);
- Предсборка: `C:\GTA6_Port\road\prebuild\` (`preflight-a.log`, `preflight-b.log`, `prebuild-keys.txt`, `prebuild.log`, `prebuild-again.log`, `snapshots.log`, `watch.log`);
- Аудиты: `C:\GTA6_Port\audit\{ambient.md,cpu.md,pr1226.md,pr1226-port.diff}` + `check-step-b.log`, `test-step-b.log`, `build-step-b.log`;
- Разведка форка: `C:\GTA6_Port\audit\recon\` (`fxceless-recon.md`, `commits-main.json`, `c-<sha>.json`, `readme-fxceless.md`, `tracker.json`);
- Бэкапы библиотеки пайплайнов: `C:\GTA6_Port\pipelines-backup-before-prebuild\` (до предсборки, 282 992 100 Б) и `C:\GTA6_Port\pipelines-backup-prebuild\` (после, 1 091 488 964 Б + prebuilt/early списки);
- Оркестраторы: `C:\GTA6_Port\run-road.ps1` (добавлен `-AsyncPresent`), `C:\GTA6_Port\analyze-road.ps1`, `C:\GTA6_Port\prebuild-road.ps1`.

## §22. Шаг 2: `VICTORY_HEAP_WORKER` (пасс текстур по индексу на своём потоке) — порт и замер с отрицательным вердиктом (2026-11-26)

### 22.1. Что перенесено

Ветка `heap-worker` от `road-to-20fps` @ `5c08c41`, коммит **`8157893`** (ff в `road-to-20fps`).
Источник: PR #1007 (`TG0000`, worker-коммит `233a65f4`), в main форка вошёл как **`d4a528ba`** («Textures picked by index can be uploaded by a thread of their own»).
Суть: пасс `State::heap_textures` (аплоад текстур, которые шейдеры выбирают по индексу) раньше шёл на том потоке, который записывает первый шейдер кадра с индексным выбором (у автора 62 мс/кадр в тяжёлой сцене и 95–116 мс на раннем сейве); теперь записывающий поток только будит воркер, воркер пишет аплоады в собственные host-листы, которые следующий сабмит выполняет перед списками титла, а слот получает вью при сабмите (`Publish`). Включается `VICTORY_HEAP_WORKER=1` (по умолчанию **выкл**).

Перенос: `heap_worker.rs` — **побайтово как у автора** (SHA-256 `E1B20BCB…3284EFD`), 288 строк; ещё 9 файлов адаптированы вручную:
- `draw.rs`: enum `Into::{List, Ahead}`, `heap_textures(Into, …)`, `kick()` вместо локального вызова в `heap_buffers`, `urgent()` для bake-пути; слияние «marked heap pass» пропущено — модуля `heap_pass` (коммит `895a23bc`, `VICTORY_HEAP_PASS_MARKED`) в нашем дереве нет, а он выключен по умолчанию;
- `texture.rs`: `Staging::refresh_ahead()` (чтение/дета́йл воркером заранее), поле `Last.ahead`, поправка в проверке повторов;
- `descriptor/host.rs`: `HostHeap::publish()`, флаг `PUBLISHING` (не ставить слот в очередь повторно), тест `a_published_slot_stays_out_of_the_queue`;
- `host.rs`: `Host::execute_late` вставляет готовые списки воркера перед списками титла (`take_ready`), `HostList::refresh_textures_ahead()`, строка в `Meter`;
- `descriptor.rs`: `view_and_key`; `object.rs`: `add_ref_alive`; `resource.rs`: `hold`/`Held`; `list.rs`: `Detached::ours`, `fresh_list` — `pub(crate)`; `lib.rs`: `mod heap_worker`.

**Зависимость:** воркер включается только при `host::late()` — т.е. при включённом relay. В нашей сборке **`VICTORY_RELAY` включён по умолчанию** (late stage 3), поэтому `VICTORY_HEAP_WORKER=1` работает «из коробки». Но **нельзя** ставить `VICTORY_LATE_WAIT=1`: это понижает стадию late до 1 (без свободного present), и прогон M4 показал **2,3 fps / 443 мс** против 4,9/205 в штатном режиме — регрессия вдвое. Флаг `-LateWait` в `run-road.ps1` оставлен только для таких экспериментов.

### 22.2. Проверки

- `cargo check -p victory-gpu --all-targets` — exit 0; `cargo check --workspace` — exit 0 (одно старое предупреждение `placed_filler_twins`);
- `cargo test --workspace` — **608 passed, 0 failed** (было 607; +1 новый тест `a_published_slot_stays_out_of_the_queue`);
- `cargo build --release -p victory --locked` — exit 0, 44,4 с; `victory.exe` 8 966 144 Б;
- два игровых прогона с воркером (M5, M5b): без потерь устройства, без паник, 31/31 кадров в фокусе, VRAM-пик 7832 и 7820 МиБ (лимит не задет), кэш шейдеров не изменился (76 973 файла), библиотека пайплайнов +1 пайплайн (1 091 553 632 Б), `prebuilt-pipelines.txt` не тронут.

### 22.3. Замеры «До/После» (1280×720, prebuilt-библиотека, ambient 0.6/1.5, штатный relay)

| Прогон | Режим | n (мир) | fps мед. | fps сред. | ms мед. | preparing | copies | copy-back | GPU-ожид. | GPU util | VRAM пик |
|---|---|---|---|---|---|---|---|---|---|---|---|
| M2 | контр. | 59 | 4,0 | 3,63 | 251,9 | 159,1 | 100,1 | 121,3 | 44,0 | 43 % | 7848 |
| M2b | контр. | 58 | 4,9 | 4,31 | 204,8 | 155,3 | 91,2 | 127,5 | 32,7 | 43 % | 7665 |
| M5 | воркер | 71 | **5,3** | **4,93** | **188,5** | **132,2** | **75,4** | **98,0** | 39,3 | 72 % | 7832 |
| M5b | воркер | 47 | 4,1 | 3,88 | 243,1 | 166,1 | 99,5 | 112,8 | 46,1 | 40 % | 7820 |
| M4 | `LATE_WAIT=1` | 44 | 2,3 | 2,21 | 442,6 | 162,9 | 122,5 | — | 246,2 | 35 % | 7834 |

При равных draws (корзина 4500–5500, медианы): M2 — 234,2 мс (preparing 161,4 / copies 100,1 / copy-back 134,2); M2b — 197,4 (152,5 / 89,7 / 132,3); **M5 — 181,0 (128,7 / 71,3 / 100,5)**; **M5b — 219,9 (165,2 / 100,4 / 122,1)**.

**Вердикт (по AGENTS п.7): механизм работает — пасс и его копии уходят с записывающего потока (M5: preparing −24 мс, copies −18 мс, copy-back −32 мс при равных draws), но выигрыш в кадре не воспроизводится:** два прогона с воркером дают 5,3 и 4,1 fps (188,5 и 243,1 мс) против двух контролей 4,0/4,9 (251,9/204,8) — разница внутри разброса прогонов. Причина честно видна из цифр: у нас пасс текстур стоит 20–30 мс/кадр (строка `heap ms`), а у автора на лодочной миссии — 96–116 мс; там же его собственный диагностический прогон 309 показывает «кадры не меняются: 9,3 против 9,1–9,6». **Оставлено в дереве выключенным по умолчанию** (нулевой риск для текущих прогонов, пригодится для лодочной сцены); **включать в основном наборе до подтверждения на той сцене — нельзя.**

### 22.4. Урок методики

Разброс между прогонами с одинаковым конфигом оказался больше эффекта: M2 4,0 против M2b 4,9 fps (Δ0,9), а разные прогоны попадают в разные сцены (worldStart 84…228 с, медиана draws 4 873…5 375). Для флагов нужен **парный замер внутри одного прогона** (ABBA, как `=alt`-переключатели форка: 20 с вкл/20 с выкл с отметками в логе, затем fit по draws). Следующий шаг — либо `VICTORY_HEAP_WORKER=alt` (добавить режим в `heap_worker::on()`), либо сразу мерить кандидатов, у которых `=alt` уже есть.

### 22.5. Что дальше (по recon §21.8, с поправкой на замер)

- Воркер оставить выключенным; проверить на лодочной сцене (нужен сейв/дальше по сюжету), где пасс 96–116 мс.
- Взять **`06702fe8`** (`VICTORY_SETTLE_ROOM=1`: −11,5 мс preparing) и **`ceb9e1f7`** (`VICTORY_PARALLEL_REFRESH=1`: −8,7/−9,4 мс) — у обоих механика ровно как у воркера (снять работу с готовящего потока); мерить только парно, иначе разброс съест эффект.
- Целиться в измеренное узкое место: **copy-back 100–130 мс и copies 90–100 мс** — попробовать `VICTORY_CARD_WINDOWS_MB=1024/512` (у автора 512 → текстуры на доме появились, кадр 154 против 137 мс — то есть хуже; 2048 дефолт), `VICTORY_BACK_READS`, `VICTORY_CARD_SYNC_BATCH=alt`, `VICTORY_LIGHT_REFRESH=alt` (у автора copies −13,7 / preparing −10,8, кадр +7).
- Для 15–20 fps нужен структурный срез, а не микро-оптимизация: смотреть в сторону меньшего объёма copy-back/ринг-буферов (это же видят в свежих коммитах форка: копии/рефреши/системные page-mapping вызовы).

### 22.6. Файлы

- Ветка/коммит: `heap-worker` @ `8157893`, ff в `road-to-20fps`;
- Дифф и аудит: `C:\GTA6_Port\audit\heapworker.diff`, `heapworker-apply.log`, `heapworker-pr.json`, `heap_worker-d4a528ba.rs`, `parent-*.rs` (исходные версии конфликтных файлов), `test-heapworker.log`;
- Прогоны: `C:\GTA6_Port\road\m4` (LATE_WAIT=1), `m5`, `m5b` (воркер), контроли — `m2`, `m2b`;
- Оркестратор: `C:\GTA6_Port\run-road.ps1` (+`-LateWait`, `-HeapWorker`).
