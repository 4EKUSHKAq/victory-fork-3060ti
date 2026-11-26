# HANDOFF — текущее состояние работ

Обновлено: 2026-11-26, 12:05 (Asia/Yekaterinburg)
Ветка: `perf-and-upstream-fixes` (от `check-env-and-build`; `master` не правился)
Ветка запушена в приватный бэкап `myfork` = https://github.com/4EKUSHKAq/victory-fork-3060ti
(коммиты: `8f12565` + коммит с этим отчётом; `master` = `d31f841`).
Ночная сессия: §12–§15 (прогоны, prefetch, чёрный кадр). Дневная сессия: §16 — бэкап-репозиторий и
разведка трёх апстримов по HDR/пересвету, теням/непрямому свету и FPS, аудит наших 254 непереведённых
шейдеров и флагов. Код не менялся, остановка перед правками — по заданию.

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
