# HANDOFF — текущее состояние работ

Обновлено: 2026-11-27 (Asia/Yekaterinburg).
Ветка: `feat-r11-1080p-ui-dlss-and-quiet` (от `feat-r10-water-heapcap-and-1440p` @ `540d9c9`).
Раунд 10 закончен: `540d9c9`, 669 passed, отчёт — §33.
**§1–§28 (раунды 1–6: окружение, команды, старые замеры) — в [HANDOFF-archive.md](HANDOFF-archive.md).** Ниже только §29+.

## §29. Раунд 7: ветка `feat-kelvaroq-audit-and-water-fix` — полный аудит kelvaroq (61 патч, 45 Group-1), порт #764 (буквальный фикс плит) + #1376 + #1365 + Belobug #287 + kelvaroq 0018/0019; прогоны 480 с: плиты на воде остались, #1371/#1377 откатаны (нет визуального эффекта, GPU wait вырос, один crash) (2026-11-27 ночь)

Ветка **`feat-kelvaroq-audit-and-water-fix`** от `feat-tiles-and-cpu-boost` @ **`9427159`**. Коммиты: **`c0d6471`** (шаг А: #764), **`22f56b7`** (#1376), **`741f531`** (#1365), **`e01bd2b`** (Belobug #287), **`9b12404`** (шаг В: kelvaroq 0018+0019), **`e61fcf6`** (#1371+#1377, откатан), **`b58bb68`** (revert e61fcf6). Итог против базы: **9 файлов, +1306/−79**; в `crates/victory-shader/**` — **0 файлов** (`git diff --name-only 9427159..HEAD -- crates/victory-shader` пуст), новых шейдеров прогоны не компилировали: кэш **76 983 файла / 3 961 774 370 Б не тронут** (ни один файл шейдерного кэша не менялся), `host-pipelines.bin` вырос **1 101 940 934 → 1 103 381 880 Б** (mtime 27.11 03:18:05) — глубокий маршрут r7a создал новые PSO; это рост библиотеки пайплайнов, не инвалидация кэша. Финальный exe **9 139 712 Б** (совпадает по размеру с билдом прогона r7a — после отката исходники те же).

### 29.1. Три read-only субагента (отчёты в `audit\`)

- **Субагент 1 — полный аудит `stellarr-studio/kelvaroq`** (`audit\kelvaroq-full-audit.md`, 72 КБ, 329 строк; 61 патч 0000–0064, gaps 0003/0007/0008/0011, есть 0064-monitor). Репозиторий — экспериментальный Xbox Series runtime на Rust (229★), собранный из **victory-fork pin `f85101b` + 61 патча + crate `native/kelvaroq-render`** (11 модулей: residency, dirty_ranges, heap_ranges, pacing, shadow_bytes, stats_ring, texel_rows, upload_pool…). Наш snapshot = `f85101b` + 39 файлов/+8 971 строк форковой работы, поэтому **~38 из 141 env-флагов kelvaroq уже есть у нас**, и «порты» части патчей = включение готовых флагов. Классификация: **Group 1 = 45** (не трогают `victory-shader`, механизм переносим без сброса кэша), **Group 2 = 7** (`0033, 0041, 0047, 0049, 0052, 0056, 0060` — меняют `victory-shader` → один отложенный round с одной пере-тёплой cache), **Group 3 = 9** (инфраструктура/инструментация kelvaroq). Top-3 Group-1: (1) `0018+0019+0025+0034` — page-mapping/alias в **`victory-hle/src/modules/xmem.rs`** (файл чист: версия pin + 5 наших строк LIGHT_REFRESH; 2–3 дня); (2) `0045-watch-reservation` (+COPY_SMALL) — **та же идея, что #1365**, поэтому порт через #1365 (шаг Б), отдельно не дублируем; (3) `0039` — floor 64 КиБ для card-buffers (env-only `VICTORY_CARD_BUFFERS_MIN_KB=64`, у нас дефолт 16 МиБ → ~3 700 из 3 800 мелких копий в RAM) + ReBAR-копии read-only памяти с бюджетом 256–512 МБ (3–4 дня, риск M на 8 ГБ). Важное: **`0044-copy-plain` у нас уже есть** (`VICTORY_COPY_PLAIN=1`), `0046/0048` (async present) у нас `present_behind` на 1 кадр; `0061-no-draw` — дешёвая атрибуция; `0062/0063` (lazy remap/UMA) — research-stage; `0060-victory-port` — не патч, а 492-строчный upstream-триаж. §6 отчёта — 8 ГБ-патчи (0017/0022/0028/0039/0053/0054/0063); §11 — 5 «бесплатных» env-A/B на уже существующих флагах (FILL_HOT, COPY_PLAIN, ASYNC_SUBMIT+QUEUE2, ASYNC_PRESENT, CARD_BUFFERS_MIN_KB=64). Group-2 план: `0047` (порядок DXIL-signature + lane-mask; GPU-часть `0x1c8/0x218` можно раньше) → `0041` (mesh launch; в victory-shader всего 16 добавленных строк) → `0033`; ключ кэша поднимать через `VICTORY_SHADER_CACHE_KEY`, старые 76 973 файла не удалять.
- **Субагент 2 — плиты на воде** (`audit\round7\water-tiles-prs.md`, 476 строк; диффы `audit\round7\pr{764,1377,1371}{,-adapted}.diff`, все `git apply --check` = 0). Механизм #764: tiled-слоты, читаемые через таблицы (tabled), `note_buffer()` намеренно **не** кладёт в `placed`, поэтому `texture_views_remade()` по `RESHOWN` их не находил; `mirror()` пересоздаёт слот только при смене ключа (`made != key`) и не замечает сдвиг `first_shown()`; вдобавок `tiles::give_back` возвращает плиты в пул **без unmap** — устаревший дескриптор читает содержимое новой владелицы («плиты чужих текстур»). #764 добавляет `has_tiled`/`tiled_shown` (by_slot + by_start), `Made { kept, tiled }`, `record_tiled/forget_tiled/tiled_stale`, ремейк в `mirror()` при `first_shown` и `placed_start()`; +2 теста. #1377 — держит depth-plane SRV дескрипторы на время жизни списка (убирает глобальное кольцо `PLANE_VIEWS=256`); #1371 — снимки RTV/DSV на `SetRenderTargets` и восстановление их при replay после разрыва списка.
- **Субагент 3 — CPU/VRAM** (`audit\round7\cpu-vram-prs.md`; `apply-check.log`): #1365 = батчинг `GetWriteWatch` по резервации памяти (новый `watch/reservation.rs` 449 строк + `watch.rs` +23 + `host.rs` +1; **off по умолчанию** `VICTORY_WATCH_RESERVATION`, лог `VICTORY_WATCH_STATS`; наш LIGHT_REFRESH-бит-путь/`publish`/`look_at_holes` не тронуты), #1376 = пере-регистрация «протухшего» тикета readback (иначе read-back дропался; пул #1359 не течёт и не дублируется), Belobug **#287** = текстура, на которую у карты не было памяти, спрашивается снова через 500 мс (`refused_until: AtomicU64`, thread-local `E_OUTOFMEMORY`, слот остаётся в очереди, `made[index]=0` при `retrying`), Belobug **#306** = script-only (`-Vram8GB`: `LIST_POOL=0`, `TEXTURE_ROOM_PERCENT=60`, `TEXTURE_BUDGET_MB=1536`; `RESIDENCY*`/`THIN_THREADS`/`CARD_AWAKE` в нашем дереве отсутствуют).

### 29.2. Шаг А — #764 (коммит `c0d6471`)

`crates/victory-gpu/src/descriptor/host.rs` +298/−23 (18 хунков, +2 теста). Адаптированный тестовый хунк пришлось поправить: тесты добавлены в `heap_size_tests`, импорт не видел `TiledSlots`/`View` — первая сборка тестов упала 3×E0425, добавлены в `use super::{…}`; после этого `cargo check --workspace`=0, **626 passed/0 failed** (было 624), release 44,4 с. #1371+#1377 (коммит `e61fcf6`) применялись чисто (+102/−41 и +72/−47; +1 тест), check=0, **630 passed/0 failed** (первая попытка — известный warp-флейк §23.3 `indirect::warp_budget…`, попытка 2 зелёная), release 40,9 с. Но по итогам прогонов r7b/r7b2 (см. 29.5) **откатаны** коммитом `b58bb68`: артефакт не исчез, GPU wait заметно вырос, один прогон упал в титульном коде.

### 29.3. Шаг Б — #1376 + #1365 + Belobug #287 (коммиты `22f56b7`, `741f531`, `e01bd2b`)

`cargo check`=0 после каждого; вместе: **627 passed/0 failed** (`watch::reservation::tests::reservation_matrix` — ignored (синтетический бенчмарк), 2 остальных reservation-теста: 1 passed/2 ignored; card-тест list.rs — ignored «needs a real D3D12 host GPU»); release 33,6 с. #287 пришлось применять частично: хунк `descriptor/host.rs` не лёг после #764 (`make` теперь возвращает `Made`, а не `Kept`) — применены `draw.rs`+`resource.rs` из диффа, хунк вставлен вручную в `keep_updates`-ветку: `built.is_none() && matches!(view, View::Shader(...) if crate::resource::retrying(...))` → `made[index]=0` (слот переспросится), а не «навсегда null view». #1365 в коде **off** (A/B отдельно), #1376 и #287 — on.

### 29.4. Шаг В — kelvaroq 0018+0019 (коммит `9b12404`)

Оба патча легли **байт-в-байт** (`git apply --check` = 0) на наш `xmem.rs` (наш файл = версия pin + 5 строк LIGHT_REFRESH): `0018-persistent-views` — один удерживаемый write-view всей секции (`whole: OnceLock`, биты committed) вместо map/commit/unmap на запрос; `0019-page-mapping-runs` — группировка подряд идущих страниц одной секции в одну view, уже стоящие маппинги не переделываются (`PAGE_MAPPINGS` счётчики). Оба флага в коде **off** (`VICTORY_PERSISTENT_VIEWS`, `VICTORY_PAGE_MAPPING_RUNS`), в харнессе с этого раунда **on**. +2 теста, **629 passed/0 failed**, release 12,5 с. Лицензия kelvaroq GPL-3.0-or-later — как наш fork, атрибуция Stellarr Studio/kelvaroq в сообщении коммита. `0044` не портировали (уже есть), `0045` не дублировали (идея = #1365), `0025/0034` не брали — их цепочка основана на другой ревизии файла (нужен rebase).

### 29.5. Прогоны 480 с (Series X, окно 1280×720 topmost, anti-AFK W/40 с, fpsbox 0, дефолты харнесса + `PERSISTENT_VIEWS=1 PAGE_MAPPING_RUNS=1`)

| прогон | дата/время | мир с | итог | ALL | WORLD | VRAM пик/сред | tiles_black |
|---|---|---|---|---|---|---|---|
| `road\r7a` (финальный исходник ветки) | 27.11 03:10–03:18 | 96 с | 480 с, 31/31 шотов, 0 ERROR, device loss нет | 222,6 мс / 4,5 fps | 253,7 мс / 4,0 fps | 7 754 / 6 762 МиБ | **653/1152 (57 %)** |
| `road\r7b` (#1371+#1377) | 27.11 03:26 | — | **crash титула на 51 с** | — | — | 3 088 МиБ | — |
| `road\r7b2` (#1371+#1377) | 27.11 03:31–03:40 | 84 с | 480 с, 30/30, 0 ERROR | 219,4 / 4,6 | 289,7 / 3,5 | 7 822 / 6 767 МиБ | 649/1041 (62 %) |
| `road\r7c-smoke` (180 с, тот же билд) | 27.11 03:27–03:30 | 78 с | 180 с, 11/11, crash нет | 191,7 / 5,3 | — | — | — |

- **r7a — контрольный прогон финальной ветки** (исходники после отката совпадают с билдом r7a): 86 frame lines; readback `copies=14464 bytes=32768 buffers_reused=15281 buffers_made=54` (пул #1359/#1376 работает); WARN 274 (как r6ab 278), не более 29 «not implemented»/225 «picked by index changed»; dropped 15 797 (r6ab тоже 16 129 — прежний счётчик). VRAM ≥7 900 МиБ — 0 сэмплов. Фазы (ALL): t≥240 **322,6 мс**, t≥300 **241,9**, t≥360 **322,6** — хуже r6ab (208,0/195,8/187,1), **но маршрут другой**: r7a median draws 2 251 против 3 271 у r6ab, зато дошёл до кадров 5 319–7 410 draws (в r6ab максимум 4 492); r6ab переразобран тем же анализатором (`audit\ab-r6ab-reanalyzed.txt`). Корзины по draws: 500–1500 **180,4 vs 230,2** (r7a лучше), 1500–3000 278,7 vs 218,6 (хуже), 3000–4500 **211,5 vs 210,4** (равно), 4500+ 584,1 (n=6) vs 250,2 (n=11) — тяжёлые кадры другого места. Абсолютное сравнение билдов невалидно (сцена/маршрут разные), выигрыш CPU-пака в игре этим прогоном не доказан.
- **Плиты: остались.** `road\r7a\shot-461_f1.png` — плиты по всей воде, `shot-476_f1.png` — полоса в дали, `shot-446_f1.png` — чистый горизонт; `road\r7b2\shot-467_f1.png` — плиты тоже видны. #764 закрыл **механизм** (тесты про remap tabled-слотов и `first_shown`), но видимый артефакт на воде не исчез → у него, вероятно, второй путь (copy-pass/copied-batch слоты не попадают в `tiled_shown` — комментарий в самом патче) или другая причина (#574/#747-класс). #1371/#1377 артефакт тоже не убрали.
- **r7b crash:** на 51 с в меню титул упал `unhandled exception 0xc0000005` в `victory-image-*.dll+0x164a298` (read 0x2c: ничего не замаплено; Breakpad-minidump, upload заблокирован), VRAM 3 ГБ, ERROR 0, кадры 1308/1309 «decoration bake». **Не детерминирован**: 180-с smoke r7c на том же билде чистый (мир с 78 с), r7b2 480 с — чистый (мир с 84 с, 30/30 шотов, 0 ERROR/device loss). Тем не менее, это второй сигнал против #1371/#1377.
- **r7b2 (с #1371/#1377) — заметно хуже**: WORLD 289,7 мс/3,5 fps; GPU wait 112,8/138,5/140,1/143,4 мс в фазах t≥180/240/300/360 (у r7a 67,9/74,9/75,4/78,4); корзина 3000–4500 **497,7** мс (n=9) против 211,5 (n=14) у r7a; 4500+ 555,0. Визуального выигрыша нет → **откат** по правилу AGENTS 7 (`b58bb68`), адаптированные диффы сохранены в `audit\round7\` для отдельного A/B (возможно, часть разницы — сцена/случайность, и crash один раз, но риск не оправдан).

### 29.6. Дефолты, харнесс, файлы

- `launch-game.ps1`: env-дефолты Round 7 **`VICTORY_PERSISTENT_VIEWS=1`**, **`VICTORY_PAGE_MAPPING_RUNS=1`**; новые переключатели **`-Vram8GB`** (пресет Belobug #306: `LIST_POOL=0`, `TEXTURE_ROOM_PERCENT=60`, `TEXTURE_BUDGET_MB=1536`; **не дефолт** — их доказательство с RX 6600M, наша потеря карты им не лечится) и **`-WatchReservation`** (#1365, off по умолчанию). Шапка описывает порты Round 7 и откат #1371/#1377.
- `run-road.ps1`: параметры `-PersistentViews 1`, `-PageMappingRuns 1`, `-WatchReservation ''` (+`VICTORY_WATCH_STATS=1` при включении), в лог/`watch.log` добавлены; контрольные плечи — пустые строки. Снапшоты: `audit\harness-launch-game-2026-11-27b.ps1`, `audit\harness-run-road-2026-11-27c.ps1`.
- Новый анализатор **`audit\analyze-run.ps1`** (frame lines, фазы, корзины по draws, VRAM пик/сред/≥7900, tiles_black, readback, ERROR/device-loss) → тексты `audit\ab-r7a-analysis.txt`, `ab-r7b2-analysis.txt`, `ab-r6ab-reanalyzed.txt`, `ab-r7c-smoke-analysis.txt` (у smoke файл не записался из-за ошибки VRAM-парсера — числа не критичны).
- Прогоны: `road\{r7a,r7b,r7b2,r7c-smoke}`; тесты: `audit\test-r7-{stepA,stepA-attempt2,pr1376,pr1365,b287,0018,0019,stepB,stepC,1371-1377,1371-1377-attempt2,final}*.log`; отчёты/диффы: `audit\kelvaroq-full-audit.md`, `audit\round7\*`; baseline кэша `audit\round7\cache-baseline.txt`.
- Тест-счётчик по шагам: 624 (база Round 6) → **626** (А) → **627** (Б) → **629** (В) → 630 (попытка с #1371/#1377) → **629** (после отката). `cargo check --workspace`=0 каждый раз; известный warp-флейк §23.3 повторился один раз (на шаге #1371/#1377), ретрай зелёный.

### 29.7. Что дальше

1. **Плиты**: #764 закрыл remap tabled-слотов, но видимые плиты остались → искать второй путь: copy-pass/`copied-batch` слоты (сделать их видимыми для `tiled_shown`/`RESHOWN`), затем #574/#747; проверить тот же артефакт на **лодочной сцене** (достижима авто-прогоном) скриншотами `shot-44x/46x/47x`.
2. **#1365 A/B**: `-WatchReservation 1` против off, 2×300 с (лог `write watch/frame` из `VICTORY_WATCH_STATS`), следить preparing/copy-back/VRAM; затем решить дефолт.
3. **#1371/#1377**: отдельный сцен-матчевый A/B (2×300 с, один и тот же маршрут) — если GPU wait подтвердится, оставить откат; диффы готовы.
4. **Group-2 kelvaroq** (отдельный round, одна пере-тёплая cache): `0047` → `0041` → `0033` с поднятым `VICTORY_SHADER_CACHE_KEY`; старые 76 973 файла/библиотеку не удалять.
5. **0039 floor 64 КиБ** — env-only A/B (`VICTORY_CARD_BUFFERS_MIN_KB=64`) на 2×300 с; ReBAR-копии (256–512 МБ) — только после, осторожно на 8 ГБ (peak 7 754–7 837 МиБ).
6. Сцена-матчевый A/B нового билда против `8eb61a6` (одна и та же лодочная сцена) — снять вопрос по GPU wait/CPU-паку.
7. Телеметрия под Series S (fpsbox в plane 0 до композита); режим — **Series X**.

## §30. Раунд 8: ветка `feat-r8-1080p-and-kelvaroq-full` — AGENTS п.8, полная телеметрия, настоящий 1080p-рендер (title-патч Belobug `VICTORY_OUTPUT`), второй путь плит, kelvaroq Group-1 (10 патчей), Group-2 (10 патчей) + мимика лиц (#927), новая cache `r8-group2`; финальный 480-с прогон: **6,5 fps ALL / 5,3 fps WORLD**, VRAM peak 6 319 МиБ, 0 ERROR/device loss; **плиты на воде остались**, чёрные лица в тёмных сценах остались (свет/тени RT — известное ограничение); ПК выключен после push (2026-11-27 ночь-утро)

Ветка **`feat-r8-1080p-and-kelvaroq-full`** от `feat-kelvaroq-audit-and-water-fix` @ **`60c7dde`**. Коммиты: **`9646679`** (AGENTS п.8), **`3fff732`** (второй путь плит), **`66419b9`** (`VICTORY_FORCE_1080P` — настоящий title-патч), **`9c0e222`** (Group-1 kelvaroq, 10 патчей), **`e9b9341`** (Group-2 kelvaroq + NVIDIA 32-lane + мимика #927, новая cache-ревизия). Итог `60c7dde..HEAD`: **39 файлов, +5 004/−845**; `crates/victory-shader/**` впервые изменён (Group-2/facial) → новый каталог кэша; старый кэш **76 973 файла / 2 614 092 344 Б** не удалён и не перезаписан.

### 30.1. Этап 0 — правила, телеметрия, дефолты

- `AGENTS.md`: добавлен **пункт 8** — в начале каждого раунда проверять 5 репозиториев (`fxceless/victory-fork`, `Marvisak/VIctory`, `stellarr-studio/kelvaroq`, `Belobug/VIctory-Belobug-AMD`, `uiliannn/VItoriaBr`) через `gh` и включать новые фиксы CPU/VRAM/рендера в план.
- `launch-game.ps1` / `run-road.ps1` (вне репозитория; снапшоты `audit\round8\harness-*-stage0.ps1`): по умолчанию теперь **`VICTORY_FPS_COUNTER=all`** (полная телеметрия), **`VICTORY_TILES_SMALL_WHEN_FULL_MB=256`** (на полной карте — уменьшенные mip-уровни вместо чёрной текстуры), **`VICTORY_WATCH_RESERVATION=1` + `VICTORY_WATCH_STATS=1`** (#1365), **`VICTORY_CARD_BUFFERS_MIN_KB=64`** (kelvaroq 0039 floor), **`VICTORY_FORCE_1080P=1`** и **`VICTORY_PERSISTENT_VIEWS=1` / `VICTORY_PAGE_MAPPING_RUNS=1`**. Контрольные плечи: `-FpsCounter 0`, `-WatchReservation ''`/`-NoWatchReservation`, `-Force1080p 0`, `-TilesSmallMb ''`, `-CardBuffersMinKb ''`.
- Проверка Stage 0: 180-с smoke `road\r8-smoke0` — 0 ERROR, `VICTORY_TILES_SMALL_WHEN_FULL_MB` дал 2 462 уровня с tiles на полной карте; строки `card buffers… least size for the card: 64 KiB` и `write watch/frame: GetWriteWatch calls…` в логе; VRAM peak 7 464 МиБ (до force 1080p).

### 30.2. Разведка (3 read-only субагента; отчёты в `audit\`)

- **Скан 5 репозиториев + 1080p + плиты** — `audit\round8\subagent1-report.md` (46 КБ), `force-1080p.diff`, `water-plates-second-path.diff`, `raw\*`. Найдено: fxceless **#927** «Mouths move» (мимика; 16-битные typed views через `emit/halves.rs`/`half_views`), fxceless **#1287** (loose-LDS + 32-lane), **#1369** (восстановление lowering-режимов после паники), Marvisak **#3124** (SDWA sign-extension), **#3348/#3406** (wave width; не включены — нужна их инфраструктура), Belobug **`VICTORY_OUTPUT`** (единственный способ реально уменьшить внутренние render-target'ы — патч кода title), VItoriaBr #72 (placed UAV aliases — не включено). Новых код-патчей kelvaroq после аудита Round 7 нет.
- **Group-1** — `audit\round8\group1\report.md`, `apply-check.log`, 10 патчей. Итог: `0025+0034` (scatter runs/page lifetime, безопасный финал), `0039` (ReBAR copies + floor/потолок), `0022` (GPU-upload inputs), `0015` (per-frame texture stage), `0016` (pipeline wait once), `0031` (root-bind cache), `0032` (relay copy runs), `0035` (queue2 landings), `0061` (`VICTORY_NO_DRAWS`). Не взяты: `0017`, `0028`, `0054`, `0053`, `0057`, `0021`, `0026`, `0002/0010/0023` (нужны модули отсутствующего crate/слишком конфликтны).
- **Group-2 + NVIDIA 32-lane** — `audit\round8\group2\report.md`, 10 патчей (0047a/b, 0041a/b/c, 0033, 0050, 0051, 0052, 0053). Плюс `audit\round8\group2-facial\report.md` — **0100/0101** (порт #927: `emit/halves.rs`, `HostShader::half_views`, half-форматы/вьюхи в дескрипторной куче, half-места в таблицах draw, encode/decode кэша). `#3348/#3406` не включены; `0102` не нужен (SDWA уже в 0053); `0103` (чёрные лица, watch-биты e4245f3c) не делался — конфликтует с `LIGHT_REFRESH`/`ABSENT_*`.

### 30.3. Шаг А — второй путь плит + настоящий `VICTORY_FORCE_1080P`

- **Плиты** (`3fff732`): `mirror_copied_batch` делал tiled-слоты, но выбрасывал `Made.tiled` → слот не попадал в `tiled_shown`; при смене `first_shown()` дескриптор продолжал читать тайлы, отданные другой текстуре. Фикс: `record_tiled`/`forget_tiled` в null/stale-ветках, GPU-gated тест `copy_pass_tiled_slot_remakes_when_first_shown_changes`. Тесты 629→630.
- **1080p** (`66419b9`): первая версия субагента (только present-скейлер) отклонена — она не создавала 1080p render-target'ы. Порт Belobug `VICTORY_OUTPUT`/`VICTORY_RENDER_SCALE` (GPL-3.0-or-later, `crates/victory-loader/src/force_1080p.rs`, 441 строка): title-патч по RVA **0x224352b** (output size), **0x247af25** (pool-check `int3` → `jmp +1`), **0x28f1e78** (own scale 2/3 → 1.0); каждый сайт сверяется побайтно, при другом build — WARN и не патчится. `VICTORY_FORCE_1080P=1` = output **1920×1080** + render scale **1.0**; `VICTORY_OUTPUT`/`VICTORY_RENDER_SCALE` — по отдельности. Наш `game_scarlett_master.exe` совпал с Belobug по всем контрольным RVA (OUTPUT/POOL_CHECK/OWN_SCALE/FFX_SCALE/PICKER/SET_SCREEN_SIZE); FFX/picker не переносились (Series S FidelityFX).
- **Проверка**: `out.log` — `resolution: settings' output size applied (1920x1080)… Done`, pool-check Done, own scale (1) Done; `err.log` — `presenting to a window width=1920 height=1080`, `display mode reported…1920x1080`. 180-с A/B (Series X, 1280×720 topmost, fpsbox all, остальные дефолты равны):

| 180-с прогон | ALL мс/fps | WORLD мс/fps | 500–1500 draws | VRAM peak/avg | swap chain |
|---|---|---|---|---|---|
| `road\r8-smoke-1080p` (force on) | **112,1 / 9,0** | **143,2 / 7,0** | **122,9 / 8,2** | **6 256 / 4 387 МиБ** | 1920×1080 |
| `road\r8-smoke-noforce` (force 0) | 163,6 / 6,1 | 185,7 / 5,4 | 177,6 / 5,7 | 7 243 / 5 199 МиБ | 3840×2160 |

  Выигрыш force: **+29 % WORLD fps** на этом отрезке, −1 ГБ VRAM peak; тесты 635 (5 новых). Визуальные различия 180-с отрезков не удалось честно привязать к флагу (разные моменты cutscene; чёрные лица встречались и до флага) — окончательный вердикт по 480-с прогону.

### 30.4. Шаг Б — Group-1 kelvaroq (10 патчей, `9c0e222`)

`0015` per-frame texture stage (`VICTORY_TEXTURE_STAGE_PER_FRAME=1`, 6 мс), `0016` `PIPELINE_WAIT_ONCE_MS=500`, `0025+0034` scatter grouping (один lock/remap-guard на ≤64 записи, view каждой страницы сохраняется), `0031` root-bind cache (`VICTORY_ROOT_BIND_CACHE=1`), `0032` relay copy runs, `0035` queue2 keep-landings, `0022` GPU-upload inputs (`VICTORY_GPU_UPLOAD_READONLY_MB=0`, A/B 64), `0039` ReBAR copies (`VICTORY_REBAR_COPIES_MB=0`, A/B 256, правило 1/10 бюджета + kept room + fallback), `0061` `VICTORY_NO_DRAWS=1` (диагностический потолок). Все новые переключатели **off по умолчанию**, поведение без них не менялось; floor/потолок card-buffers уже были в дереве и включены env-дефолтом `VICTORY_CARD_BUFFERS_MIN_KB=64`. Два тестовых фикса: инициализатор `Shader` в новом root-bind тесте (ambient/walks) и `#[cfg(test)]` у `shadow::register`. Тесты **642 passed / 0 failed**, `cargo check`=0.

### 30.5. Шаг В — Group-2 + NVIDIA 32-lane + мимика (#927), новая cache (`e9b9341`)

- **0047a/0047b**: GPU-метод `0x218` (precompiled compute), порядок элементов DXIL-signature по offset semantic-имени, скалярная копия EXEC как данные в lane-branch, логи wave width/refused стадий; **0041a** `CARD_HASH_WIDE` (64 потока на 64-КиБ кусок); **0041b** mesh payload 4 Б в PSV; **0041c** **mesh launch amplification** (`launch.hlsl/launch.rs/launch_code.rs`, бюджет списка, прямые `DispatchMesh`, plain-двойник для indirect; **on by default**, контроль `VICTORY_MESH_LAUNCH=0`); **0033** provably-empty wave64 mask; **0051** fxceless #1287 loose-LDS (off, `VICTORY_WAVE_LDS_LOOSE=1`) + точные GroupInfo/s_pack/s_bfm/s_mov/f64/lane-mask; **0052** #1369 lowering modes after panic; **0053** Marvisak #3124 SDWA sext; **0100/0101** #927 facial halves (рот/морфы: сеть лиц больше не читает нули). Новый каталог кэша **`<cache>/shaders-r8-group2`** (`0050`, `VICTORY_SHADER_CACHE_REVISION=0` — откат на старый `shaders`), соль ключа **2 → 3**; старый кэш не тронут. Тест-модуль `lower.rs` (23 КБ) вынесен в `emit/lower/tests.rs`, чтобы файл прошёл asset-policy 512 КиБ; тест `only_a_provably_empty_mask…` скорректирован под 0047b (копия EXEC через `s_or` теперь читается как данные). Тесты: **658 passed / 0 failed** при `cargo test --workspace -- --test-threads=1`; параллельный запуск GPU-тестов один раз упал `0xc0000005` (серийный — чистый), известный warp-флейк ретраился.
- **Прогрев**: `target\release\victory.exe translate C:\GTA6_Files\shaders.rpf --warm all --jobs 12` (VICTORY_HOST_WAVE=32, VICTORY_VIEW_LIMIT=999999): **25 889 хэшей, 25 638 переведено, 251 не переводится; 229,2 с на 12 потоках**. Новый кэш: **25 641 файл / 872 069 095 Б**; старый **76 973 / 2 614 092 344 Б** не изменён. `host-pipelines.bin`: **1 103 444 436 → 1 180 542 530 Б** (новые mesh/amplification/plain PSO дописаны; не удалён).
- **Смоук 180 с** `road\r8-shader-smoke`: 0 ERROR/device loss; mesh launch реально работает (`launched=true`, до 4 131 групп/кадр, 2 004 записей/кадр); cache hits 2 396 / misses 2. Первые world-кадры — **2,1 fps** из-за создания ~1 000 новых PSO; к концу 6 fps. Это ожидаемая цена первого запуска новой шейдерной библиотеки.

### 30.6. Финальный контрольный прогон 480 с (`road\r8final`)

Series X, окно 1280×720 topmost, anti-AFK W/40 с, **все дефолты Раунда 8**: `FPS_COUNTER=all`, `TILES_SMALL_WHEN_FULL_MB=256`, `WATCH_RESERVATION=1+STATS`, `CARD_BUFFERS_MIN_KB=64`, `FORCE_1080P=1`, `PERSISTENT_VIEWS=1`, `PAGE_MAPPING_RUNS=1`; новый кэш + прогретая pipeline library. `worldAt=75`, **30/30 скриншотов**, 480 с, 0 ERROR, 0 device-loss, процесс закрыт чисто. Заголовок/`err.log`: `presenting to a window width=1920 height=1080`; title-патчи resolution Done.

| метрика | значение |
|---|---|
| ALL | **153,1 мс / 6,5 fps** (n=84) |
| WORLD | **189,5 мс / 5,3 fps** (n=70) |
| WORLD t≥180 / ≥240 / ≥300 / ≥360 | 204,0 / 4,9 · 245,6 / 4,1 · 239,4 / 4,2 · 189,7 / 5,3 |
| корзины draws 500–1500 / 1500–3000 / 3000–4500 / 4500+ | 145,9 / 6,9 · 204,0 / 4,9 · 344,3 / 3,0 (n=10) · 192,0 / 5,6 (n=12) |
| draws median | 2 026 |
| VRAM peak / avg / ≥7 900 МиБ | **6 319 / 5 495 МиБ / 0** |
| `tiles_black` | 432 из 868 (50 %); small-tiles дано 8 409 уровней |
| readback | copied=14 592, buffers_reused=15 464, buffers_made=47 |
| dropped | 12 216 (r7a 15 797) |
| mesh launch | 1 618–4 034 записей/кадр, 217–430 count'ов на CPU/кадр |

Сравнение с Round 7 `road\r7a` (222,6/4,5 ALL, 253,7/4,0 WORLD, peak 7 754/avg 6 762, `tiles_black` 653/1152, dropped 15 797) — маршрут другой (median draws 2 026 против 2 251, но состав кадров иной), однако по сопоставимой корзине 500–1500: **145,9/6,9 против 180,4/5,6**; 1500–3000: **204,0/4,9 против 278,7** (r7a хуже); 3000–4500: 344,3/3,0 против 211,5/2,8 (хуже, n=10/14). Итог: абсолютный fps выше, VRAM заметно ниже, тяжёлые кадры местами хуже — нужен сцен-матчевый A/B.

### 30.7. Визуальный итог (честно)

- **Лица**: на `shot-420/450/465` лица детальные, кожа/одежда текстурированы, на 420 виден открытый рот — мимика/морфы оживают (#927 сработал). Чёрные лица в тёмных сценах остаются (`shot-170`: глаза/контур видны, лицо чёрное) — это известная нереализованная RT-подсветка/тени (README: «light in shadow missing»), не регрессия #927; watch-часть e4245f3c не переносилась.
- **Плиты на воде — ОСТАЛИСЬ**: в `shot-465` (лодка, вода на фоне) и `shot-450` видны тёмные прямоугольники чужих уровней; второй путь (copy-pass tiled slots) механизм закрыл, но видимый артефакт не убрал → третий путь/причина: динамические `fresh_tables` (`draw.rs`), #574/#747 или `VICTORY_LEVELS_ON_DEMAND=0` A/B.
- **Свет/overexposure**: на 170 тёмная сцена почти чёрная (лицо), на 420–465 нормальная экспозиция; вылетов нет, «мыльных» текстур вблизи не видно; один кадр `t=310` — 2 000 мс (создание PSO/загрузка), к 360 с стабильно 5,3 fps.
- **No device loss** за 480 с; VRAM с запасом ~1,8 ГБ до 8 192; `buffers_made` 47 — пул readback не течёт.

### 30.8. Дефолты, файлы, откат

- Лаунчеры (вне репо, снапшоты `audit\round8\harness-*-stage0.ps1` и финальные `audit\round8\harness-*-final.ps1`; актуальные копии — `C:\GTA6_Port\launch-game.ps1`, `C:\GTA6_Port\run-road.ps1`): новые дефолты из 30.1; `launch-game.ps1` — `-NoWatchReservation`, `-Force1080p`, `-TilesSmallMb`, `-CardBuffersMinKb`; `run-road.ps1` — те же параметры + `-WatchReservation ''`/`-Force1080p 0` контроль, fpsbox остаётся `all` для мониторинга.
- Откат шейдерного пакета без потери кода: `VICTORY_SHADER_CACHE_REVISION=0` (старый `<cache>/shaders`), `VICTORY_MESH_LAUNCH=0`, `VICTORY_CARD_HASH_WIDE=0`, `VICTORY_WAVE_ZERO_MASK=0`, `VICTORY_WAVE_LDS_LOOSE=0`; откат 1080p — `-Force1080p 0`/`VICTORY_FORCE_1080P=0` (title не патчится).
- Артефакты: `audit\round8\subagent1-report.md`, `group1\{report.md,apply-check.log,NNNN-*.patch}`, `group2\{report.md,apply-check.log,NNNN-*.patch}`, `group2-facial\{report.md,0100,0101}`, `warm-cache.log`, `r8final-console.log`, `ab-r8{,-smoke0,-smoke-1080p,-smoke-noforce,-shader-smoke,-final}-analysis.txt`, `test-*.log`, `build-*.log`; прогоны `road\r8{,-smoke0,-smoke-1080p,-smoke-noforce,-shader-smoke,-final}`.
- Тесты: 629 (конец Round 7) → 635 (force 1080p) → 642 (Group-1) → **658** (Group-2/facial), `cargo check --workspace`=0; полный прогон — серийный (`--test-threads=1`), параллельный GPU-тест-бинарь один раз упал `0xc0000005`.

### 30.9. Что дальше

1. **Плиты на воде**: проверить `VICTORY_LEVELS_ON_DEMAND=0` (A/B 2×300 с) — исключить tiled-путь; затем динамические `fresh_tables` в `draw.rs` (свежие таблицы не пишут `made[]`/`tiled_shown`) и #574/#747; смотреть `shot-4xx` на лодке.
2. **Чёрные лица/свет**: перенести watch-часть e4245f3c (`VICTORY_PAGE_WRITES 1|2|8`, `texture_slots_page_written`) отдельным Group-1 шагом с A/B; затем ambient/RT-свет (известное ограничение README).
3. **Mesh launch**: A/B `VICTORY_MESH_LAUNCH=0` на 2×300 с; первый запуск после новой cache создаёт ~1 000 PSO и стоит 1–3 fps — для игровых сессий прогреть pipeline library `victory translate --cache`/prebuild или оставить один прогревочный запуск.
4. **251 refused shader / 3 compute pipelines** (`2da4beb386509b77`, `6b82bc7d22016cb5`, `02384a60fda6f2d0`): проверить `VICTORY_WAVE_LDS_LOOSE=1` на этих трёх (loose-LDS off по умолчанию), затем #3406/#3348 отдельным раундом.
5. **Group-1 env A/B**: `VICTORY_ROOT_BIND_CACHE=1`, `VICTORY_TEXTURE_STAGE_PER_FRAME=1`, `VICTORY_GPU_UPLOAD_READONLY_MB=64`, `VICTORY_REBAR_COPIES_MB=256` (осторожно на 8 ГБ) — по одному флагу, 2×300 с.
6. **Сцен-матчевый A/B** против `8eb61a6`/`r7a` (лодочная сцена, один маршрут), чтобы отделить эффект шейдерного пакета от разницы маршрутов; проверить `VICTORY_MESH_LAUNCH=0`/`CARD_HASH_WIDE=0` при просадке тяжёлых кадров.
7. **Series S**: композита двух plane'ов в этой ветке нет (нужен мерж `feat-series-s-upscaler` #798/#1081); force-1080p на Series S — output 1080p, но FSR4-путь по-прежнему чёрный (pre-existing).
8. Ветка запушена в `myfork` (`https://github.com/4EKUSHKAq/victory-fork-3060ti/tree/feat-r8-1080p-and-kelvaroq-full`, PR-create: `.../pull/new/feat-r8-1080p-and-kelvaroq-full`), `git push` exit 0, remote SHA сверен; режим — **Series X**. После успешного `git push` — штатное выключение ПК (`shutdown /s /t 60`).

## §31. Раунд 9 — research (код не менялся): покрытие **7/7** звёзд (`xcl` + `XSEmu` впервые), дельта fxceless `9a9091d1` («Ray tracing runs»), DXR-док, CPU-аудит coolfox; сводный план Раунда 10

Разведка 2026-11-27 05:00–06:20Z (10:00–11:20 +05:00). `crates/` **не менялся**, `HEAD 84df20a`, рабочее дерево чистое, тесты не запускались (658 passed с Round 8 остаются в силе), шейдерный кэш не тронут. Все отчёты — в `C:\GTA6_Port\audit\round9\`.

### 31.1. Покрытие и агенты

- `gh api user/starred` = **ровно 7 репозиториев** (других звёзд нет): `fxceless/victory-fork`, `Marvisak/VIctory`, `stellarr-studio/kelvaroq`, `Belobug/VIctory-Belobug-AMD`, `uiliannn/VItoriaBr`, **`xcl-dev/xcl`**, **`zortos293/XSEmu`**.
- Первый проход Round 9 покрыл только 5 из AGENTS п.8. `xcl` изучался лишь в Round 16 (§16.6, один reference-PR), `XSEmu` **не упоминался ни разу**. Пробел закрыт в этом раунде; заодно перепроверена дельта по 5 репо.
- Агенты и отчёты: `r9-fx-marvisak` → `r9-fx-marvisak.md` (35 КБ), `r9-bel-vitoria` → `r9-bel-vitoria.md` (39 КБ), `r9-kelvaroq-scan` → `r9-kelvaroq-scan.md` (36 КБ), `r9-dxr-doc` → `r9-dxr-doc-study.md` (84 КБ), `r9-refresh5` → `r9-refresh-5.md` (16 КБ), `r9-xcl` → `r9-xcl.md` (39 КБ), `r9-xsemu` → `r9-xsemu.md` (42 КБ), фоновый `coolfox-perf` → `C:\GTA6_Port\coolfox\perf\audit-top-thread.md` (8 КБ). Сырые артефакты: `raw\{xcl,xsemu,refresh5}\` — 143 файла, ~11 МБ.

### 31.2. Пять известных репо — что берём (перенос из первого прохода)

| Приоритет | Источник | Суть | Применимость / статус |
|---|---|---|---|
| **P0** | fxceless `81d54b06f7` (PR #1407 CLOSED, правка влита в main) | `shadow.rs`: `(at - (start+at) % PIECE).max(from)` → `saturating_sub`; big-compare ≥16 МиБ не сравнивался | 1 строка, **наша строка на месте**; правка без кэша |
| **P0** | fxceless #1380 (merged 26.11) | `clzero` на guarded/uncommitted линии — крах `0xC000001D` | main-код ложится, отвергается только тестовый хунк |
| **P0** | env A/B | `VICTORY_TILES_SMALL_WHEN_FULL_MB` **256 → 512** (upstream default; r8final `tiles_black` 432/868) | без кода, 2×300 с |
| **P0** | fxceless `a91816384` LDS init hoist (в main с 03:39Z) | вода: min/max init пропускался на early-return → тёмные пятна | ~200 строк + **re-warm cache**; дешёвый пред-A/B — не гнать два водных хэша `5dc3b259…`/`6f225b64…` wave-wide |
| **P1** | fxceless #1392 (merged 26.11) | текстура «partly mapped, never filled» остаётся нулями при `TEXTURE_NO_WAIT=1` (наш default) | ручной порт |
| **P1** | VItoriaBr #72 | placed-UAV aliases (2D/array/3D) | `apply --check` чистый, кэш не нужен; тест вода/H-B |
| **P1** | Belobug #449 / #414 | `legal_state` для обеих сторон barrier / `FRAME.load` до swap | apply clean, без кэша |
| **P1** | Belobug #396 / #470, VItoriaBr #59 | перепроверка in-place текстур после ~572 кадров / detile-retry / mesh `exp prim` 9-бит | ручной порт; #59 — re-warm |
| **P1** | fxceless `ff1776b79` | `VICTORY_DUMP_REFUSED` — дамп отказов (15 строк, диагностика) | в main, пушем не тронут |
| **P2** | fxceless #1419 / masked patch/lean (#1352 merged, −13 мс card) | paired wave64 / ~490 строк, конфликты | отложено |
| watch | Marvisak / kelvaroq | merge нет; у Belobug 5 новых open-PR #476–#480 (WC-копии, write guard, geometry, adaptive pools) | только наблюдение |

### 31.3. fxceless — дельта **после** отчёта (refresh5)

- `main`: `674290601` → **`9a9091d180f0`**, 2026-11-27T05:28:07Z, «Ray tracing runs: the structure builder no longer stalls the title; rays see the structures» (10 файлов, +522/−59, без PR). Внутри: (1) sync title-памяти по адресам — каждый список синхронизирует только копии своих dispatch'ей, лимит 1023→4095, `VICTORY_MEMORY_SYNC_ALL`; (2) written-page notes — store/atomic помечают 64-КиБ страницу, назад копируются только они (`shadow.rs::pull_back`); (3) RT-адрес в `Location` view (было 0 → пустой узел); (4) instance/procedural-ноды → all ones как AMD GPURT.
- **Следствие:** краш `XboxBvhBuilder.dll` через 30–40 с — это **watchdog титула** (запись в 0), а не битый указатель; **RT-тени/свет теперь видны**, но **0.2–3 fps** (до 8 ГБ копий/кадр, ~150 мс CPU). README-статус 06:30Z. Для нас: BVH/RT — **watch only**, в Round 10 не брать (нужны paired wave64 + фикс 3 refused + memory-датапас; у апстрима fps тоже неприемлемый).
- Статусы: #1380/#1392/#1352 merged 26.11 23:06Z; **#1407 CLOSED без merge**, но правка в main коммитом `81d54b06f7` (`at.saturating_sub(...)` — проверено по файлу); `ff1776b79`, `a91816384` в main с 03:39Z. Новые микро-CPU-кандидаты: #1436 (`stage_spans` Vec alloc), #1427 (`Shadow::card_merge_relay`).
- Остальные 4 без кода: HEAD'ы совпадают со снимками отчётов — Marvisak `63bcca43c6`, kelvaroq `e062a82e04`, Belobug `4eb6cfdf9e`, VItoriaBr `1361e190b7` (у VItoriaBr `pushedAt` = **26.11 04:09Z**, не 27.11; после 04:00Z активности нет).

### 31.4. `xcl-dev/xcl` (GPL-2.0, Rust; 8 639★, 1 204 форка, создан 2026-11-20 — вся история 7 суток; HEAD `c2ee10e4e905`, 1 587 open PR, 333 issue)

Прямой перенос кода **запрещён лицензией** (GPL-2.0-only несовместим с нашим GPL-3.0-or-later) — только идеи, алгоритмы, цифры. Законный код — `OpenScarlett` (MIT, C++20, `dxr_hardware.cpp`) и `locum` (MIT, Rust, capture/replay с `verify --times`).

1. **№1 под наш copy-back — `XCL_MAP_ASYNC` (#2952 п.5) + #1993.** Их профиль на NVIDIA — наш 1:1: GPU занят 52–56 %, CPU-bound, потоки тайтла стоят в `Map → sparse::readback_now → wait_own_fence`, ~24 000 `Map`/10 с; каждый `Map` писал свежий copy-back и **ждал его на CPU**. Фикс «ждать только перекрывающиеся копии»: **5.5 → 7.6 presents/s (+38 %), GPU busy 52 → 78 %**. Это прямой аналог наших `copy-back 55–76 мс` при GPU busy 56–69 %.
2. **`WaveSize(64)` у них решён, а не отказан** — `docs/wave64-on-wave32.md` (25,5 КБ, в main): на адаптере с `WaveLaneCountMax<64` (наш RTX 3060 Ti, Ampere 32..32) `[WaveSize(64)]`-PSO отвергается при создании — **ровно наши 3 отказа**. Решение: `[WaveSize(32)]` + две hardware-половины + обмены; **split** (у нас уже есть в `emit/lower.rs:1344+`) и **packed** (1 host-wave32, lane i = console-lanes i и i+32 — **формы у нас нет**); liveness масок снижает worst-case **3.90–4.14× → 1.07–1.16×**. Риск: #1727 GPU hangs на RTX 5070; #2964 fail-closed по таймауту. Бонус — разблокирует наш `VICTORY_BVH_BUILDER`.
3. **Пайплайн-шторм: рычаг — размер DXIL, не воркеры.** Cold-компиляция ~`N^1.47`, 12 driver-слотов заняты, ~4 МБ DXIL ≈ 951 с; #2080 (merged): split по CFG-границам → **14–23 с → 2.2–2.5 с** ценой 1.57–2.06× runtime. Политика: `XCL_HUGE_PSO_CONCURRENCY=1` (огромные PSO **по очереди**), ≥4 МиБ — one-wide lane, гейт по 3 ГБ commit, потолок `XCL_PIPELINE_MAX_KB`=8 МиБ. Наши «14 пайплайнов по 5–7 с, wanted at once» — вероятно параллельная компиляция, душащая драйвер.
4. **Подготовка до старта + whole-form-first** (#2555 merged, #2952 п.3): whole-форма split-программы строится **первой** → «0 chained dispatches в каждом world-прогоне»; иначе **468 с ожиданий на 20 формах**. #2371: offline whole-pipeline blobs → **4.07 → 0.52–0.60 с** на PSO, smoke 1020 → 1266 кадров.
5. **VRAM: готовая политика 75/85/92 %** (#2385/#2968): по заполнению бюджета кап стороны 2048/1024/512 + mip-clamp через `ResourceMinLODClamp` (tiled tier 2 читает нули), RT/depth/UAV не трогать. На 8-ГБ карте pages **3.2 → 2.29 ГБ**, VRAM **10.2 → 9.0 ГБ**. Наши 7.12/8.19 ГБ — их «>85 %» бакет; чёрные тайлы лечатся clamp'ом, а не отказом.
6. **Наши host-copies, вероятно, не memcpy, а цена imported heap.** #2844 (merged, 4.9× FPS): GPU-loads/atomics на `OpenExistingHeapFromAddress` на NVIDIA в **8–22× медленнее** driver `CUSTOM` L0 heap (проба 573 vs 44.7 нс/итер.); окно в CUSTOM L0 → **0.62 → 3.03 FPS**, mixed-окно #2938 → **0.58 → 25.28 FPS (43×)**, PCIe −2/3. CPU-чтения BAR шли 12–84 МиБ/с → читать через COPY-очередь.
7. **Модель стоимости «лука» (#582):** `GetWriteWatch` = **1.15 мкс + 0.45 мс/ГиБ**; план (64-КиБ read-only блоки + vectored handler) → **0.83 → 0.02 мс**. Если наши 55–84 мс — скан «что изменил CPU», цель — единицы мс.
8. **RT: xcl остаётся на CPU-эмуляции узлов BVH, host-DXR не реализован** (#1751 — проект с разобранными `BATCHED_DESC2`/rehydrate/layout). Ключевое: их `XboxBvhBuilder.dll` **запускается нативно**, наша нет → развилка «собрать console-BVH самим» либо host-DXR, не быстрый фикс. Ещё #1412: отказы надо кэшировать по capability («boot pays for the same refusals every run») — про наши 3.

**Вывод:** xcl даёт существенно новое против §16.6 — три идеи доехали до main с замерами (#2844, #2555, #2371), #2938 показывает запас 43×, появились спека wave64 и модель write-watch.

### 31.5. `zortos293/XSEmu` (GPL-2.0-only заявлена, но 6 файлов GPL-3.0-or-later; Rust 11.7 МБ / 313 `.rs`; 732★, создан 2026-11-23, HEAD `439a152322`, 30 коммитов, 12 merged PR, 0 релизов, 4 шумных issue)

- **Тот же класс и та же игра:** Wine-подобный HLE-раннер Xbox Series X|S, ответвление `ivall/xcl`; дефолтная цель — **GTA VI, TitleId `7952987A`** (`hle/d3d12x/title_gta6.rs`), прогоны `play-45…play-98`, мир на `play-87n`; тот же `shaders.rpf` (26 503 записи).
- **RT у них принципиально иначе:** host DXR не используют, RT по умолчанию **выключен** (`RaytracingTier`=0, `CreateStateObject`→`E_NOTIMPL`), билдер all-or-none; GTA VI никогда не спрашивает `OPTIONS5`. Наши RT-проблемы они **не решают**; ценность — память/VRAM/волны/пайплайны.
- **P0 `docs/wave64-on-wave32.md`** — полная спека split+packed, обмены, барьеры/счётчики, hazards, `upper_lanes` (13 210 wave64 VS/PS, 749 программ с lane≥32), liveness масок; подтверждает наш отказ 3 CS и блокировку BVH-билдера.
- **P0/P1 `image_writeback.rs`** («ZOR-603») — точный ленивый page-guard writeback GPU-записанных изображений: авторитет = фактическая сабмиссия, захват физстраниц, guard на адрес и алиасы, отдача только по CPU-чтению, тайлинг в консольный layout, 23 счётчика отказов. Это **порт нашего же `texback.rs`** (fxceless PR #999) с улучшением; файл GPL-3.0-or-later ⇒ **обратный перенос законен с provenance**. Их пример — **water height map, по которому плавают лодки** ⇒ прямо в нашу «воду».
- **P1 VRAM/тайлинг:** 64-КиБ страницы + алиасинг таргетов по одному адресу + lazy tiles только там, где тексели (retail 5.2 ГБ текстур над 2.9 ГБ страниц) + 1-МиБ slabs (−0.9 ГБ) + residency/evict; их баг «3.5 ГБ адресов → 15 ГБ арен → device removed» — готовые грабли для наших 7.12/8.19 ГБ, чёрных тайлов и placed-UAV alias; GFX10 tiling XOR/PipeConfig (909/912 текстур байт-в-байт) — эталон сверки.
- **P1 пайплайн-шторм:** present-chain-first (`FFX_*` = 27 FSR4 + RCAS + SPD; «2 из 29 собраны на 451 с за 1 547 ожидающими»), early shaders, stand-in+воркеры+бюджеты, admission (giants ≤3, `30+10·MiB`), манифест+warm-up. У нас **уже зрелее** pipeline library и воркеры ⇒ брать нужно приоритизацию, а не механизм.
- **Лицензия:** из `crates/xcl/**` — только идеи/факты/форматы с атрибуцией; из 6 GPL-3-файлов (`image_writeback`, `title_gta6`, `pso_manifest/sched/library`, `warm_children`) возможен прямой перенос с provenance-нотой. `title_gta6.rs` целиком не цитировать («Not for a public or shared build»).
- **XSEmu смотрит в наш репозиторий:** PR #999 → `image_writeback.rs`; CGBJ `f4b48ad` → 6 файлов; их Qt6-лаунчер собирает/запускает `avatarmanz/VIctory-7900XTX`, `DexNom/VIctory`, `kelvaroq` v0.2.0, `cool_fox` @`f4b48ad` и парсит прогресс наших CLI (`victory shaders`, `warm --used`); готовый A/B-набор из 17 `VICTORY_*`-флагов «самой быстрой» конфигурации (9 из 17 уже есть у нас; `RELAY_COPY`, `CARD_STEPS`, `INDIRECT_BATCH`, `MAP_THREADS`, `CARD_AWAKE`, `SHOW_AFTER`, `THIN`, `UPLOAD_AHEAD` — только у cool_fox; `THIN` — NO-GO, см. 31.6).
- **Вывод:** включать в мониторинг **третьей линией** после Marvisak/fxceless/kelvaroq — единственный, кроме нас, проект, ведущий GTA VI на близком стеке и публикующий решения с числами. Роль — референс-архитектура и чек-лист, не копипаста.

### 31.6. Фоновый `coolfox-perf` — CPU-аудит top-thread (завершился в этом раунде)

Замеры по `play-r8` (наш мир, 3060 Ti): кадр медиана **231,9 мс**; preparing (`victory-submit`, ASYNC_SUBMIT=1) **112,9 мс**, из них copies **80,6 мс / 131–144 МБ**, textures 12,9; лендинг relay **99,7 мс/кадр** (p90 207,5, max 1079); relay ожидание карты/фетчи 27,0/8,3; GPU 33,4 / copy-back wall 71,5; TOP THREAD (stats.rs) медиана **66 %**, p90 **100 %**, ≥87 % в 18,7 % проб; settle-ожидания **0,0 мс**; heap-pass 14,9 (в тяжёлых 42–86).
- **AVX2-компаратор coolfox — не брать отдельно:** у нас `changed_runs` SSE2, медиана 714 runs / 0,67 МБ/кадр ⇒ выигрыш <1–2 мс; их выигрыш от «pairs dispatch», не от AVX2.
- **Дельта-возврат (`VICTORY_LAND`) — главный рычаг:** у нас `Shadow::card_merge_relay` на различающийся кусок **читает весь stretch из title-памяти** и пишет обратно весь stretch; многопоточно только сравнение. Точечный порт (без их asm/`land.rs`): считать runs «gpu vs held» и писать только их — **~60–120 строк `shadow.rs`** + флаг + тесты; второй шаг — 3–4 потока (~150 строк `host.rs`). Ожидание: relay-лендинг **~100 → 30–40 мс** (p90 207 — больше), VRAM **0**.
- **PAGE_PARTS (`3ead56e`):** копировать только записанные 4-КиБ части 64-КиБ страницы («29 МБ записано, копировалось 114 МБ») — наш профиль copies 80,6 мс / 131–144 МБ; ~200 строк `watch.rs`+`shadow.rs`.
- `thin.rs` — **NO-GO**: это не наш top-поток, отдельная ветка 1,7 тыс. строк, их же прогон на 3060 Ti дал 2–3 fps против наших 6.
- **До порта — атрибуция top-потока (без кода, 3 пары по 120 с):** `RELAY=2↔0`, `ASYNC_SUBMIT=1↔0`, `HEAP_WORKER=1↔0`; плюс 20 строк — писать tid победителя в `stats.rs`. Нулевые A/B: `VICTORY_COPY_CHANGED=1|alt` (есть, off), `VICTORY_MERGE_GAP=0` vs 64.
- Всё — только `crates/victory-gpu` (CPU-side), `victory-shader` не трогается ⇒ кэш остаётся валидным; новых device-аллокаций нет.

### 31.7. DXR-документ (`r9-dxr-doc-study`)

Не общая статья «CPU одно / GPU другое», а проект native host DXR для `victory-corgi` (RTX 5090): BLAS/TLAS + DXIL `RayQuery` вместо консольной эмуляции в 64 compute-программах. Подтверждает: наш RT **CPU-bound** из-за host-копий title-памяти; RT-ядра GPU не используются. Но crux — замена traversal-loop в 64 программах (стадия 3 — офлайн-survey, go/no-go), недели работы; у нас 8 ГБ (пик 7.12/8.19, запас ~1 ГБ), builder на NVIDIA нестабилен, нашего `native_rt.rs`/RayQuery/DXR-типов **нет**. **Native DXR сейчас не планировать.** Дешёвые шаги: офлайн-survey 64 `image_bvh*`/live-outs; `VICTORY_BVH_HOLD=1` A/B; короткий builder+quarantine с DRED; счётчики by-address. Поправка: 3 refused compute — **не builder** (см. 31.10).

### 31.8. Сводный план Раунда 10 (после всех источников)

| # | Приоритет | Что делать | Источник | Ожидаемый эффект | Риск |
|---|---|---|---|---|---|
| 1 | **P0** | copy-back не ждать на CPU: аналог `XCL_MAP_ASYNC` (ждать только перекрывающиеся копии) | xcl #2952 п.5 / #1993 | наш copy-back 55–76 мс ↓; GPU busy ↑ | средний |
| 2 | **P0** | `shadow.rs` compare `saturating_sub` | fxceless `81d54b06f7` | корректность big-compare ≥16 МиБ | низкий |
| 3 | **P0** | env A/B `TILES_SMALL_WHEN_FULL_MB` 256↔512 | Round 9 | `tiles_black` 432/868 ↓ | нулевой |
| 4 | **P0** | wave64 → 3 refused CS: начать с дешёвой ISA-классификации, затем **packed**-форма + liveness масок (split уже есть) | xcl doc + XSEmu doc | разблокирует 3 CS и BVH-билдер; 1.04–1.16× overhead | высокий |
| 5 | **P0/P1** | точный ленивый writeback по авторитету сабмиссии (`texback` → submission-authority+guard) | XSEmu `image_writeback.rs` (GPL-3, производный от нашего #999) | copy-back точнее; вода/height map | средний |
| 6 | **P1** | VRAM-политика 75/85/92 % → cap 2048/1024/512 + mip-clamp | xcl #2385/#2968 | чёрные тайлы, пик 7.12/8.19 | средний |
| 7 | **P1** | дельта-лендинг (писать только различающиеся runs) + PAGE_PARTS | coolfox-perf | relay 100 → 30–40 мс; copies −объём | средний |
| 8 | **P1** | проверить imported heap vs `CUSTOM` L0 (8–22× медленнее GPU-loads) | xcl #2844/#2938 | возможно, объясняет наши copies | низкий (A/B) |
| 9 | **P1** | present-first приоритизация PSO (`FFX_*`/RCAS/SPD) | XSEmu `title_gta6` | −50…−80 % first-frame на 14 PSO | средний |
| 10 | **P1** | мелкие чистые порты: Belobug #449/#414/#396/#470, VItoriaBr #72/#59, #1380, #1392 | Round 9 | корректность/стабильность | низкий |
| 11 | **P1** | сериализация huge-PSO + потолок DXIL (`XCL_HUGE_PSO_CONCURRENCY`, `XCL_PIPELINE_MAX_KB`=8 МиБ) | xcl #2080 | пайплайн-шторм 14×5–7 с | средний |
| 12 | **P2** | `ff1776b79` `VICTORY_DUMP_REFUSED`, #1436/#1427 микро-CPU | fxceless | диагностика | низкий |
| 13 | watch | RT: fxceless `9a9091d1` (0.2–3 fps), native DXR, XSEmu RT-off bench (`VICTORY_RT=off`), 17-флаговый чек-лист | xcl/XSEmu/fxceless | понимание цены RT | — |
| 14 | NO-GO | `thin.rs`, AVX2-компаратор отдельно, RT-раунд сейчас | — | — | — |

### 31.9. Артефакты

- Отчёты: `audit\round9\r9-{fx-marvisak,bel-vitoria,kelvaroq-scan,dxr-doc-study,refresh-5,xcl,xsemu}.md`; CPU-аудит: `C:\GTA6_Port\coolfox\perf\audit-top-thread.md`.
- Сырое: `audit\round9\raw\` — 472+ файла (repo evidence, диффы, `dxr-doc.md`, `xcl\` 50 файлов, `xsemu\` 52 объекта + tarball HEAD, `refresh5\` 51 файл).
- Игровая сессия пользователя: `C:\GTA6_Port\play-r8\` (596 с, 16/38 удачных кадров, пик VRAM 7 121 МиБ, 0 ERROR/device-loss).

### 31.10. Поправка к §30.9 п.4 и следующий шаг

- **Поправка:** три refused compute-пайплайна (`2da4beb386509b77`, `6b82bc7d22016cb5`, `02384a60fda6f2d0` = `alphaResolveCS_AlphTransmissionPrefilterMip`, `vfx_triangleClipCS_CollectTriangles_Meshlets`, `vfx_triangleClipCS_DefragHeap_ComputeRelocations`) — это **не XboxBvhBuilder**, а `WaveSize(64)`-отказ `CreateComputePipelineState` из `shaders.rpf`. Теперь подтверждено спекой wave64 (31.4/31.5): на 32-полосном адаптере `[WaveSize(64)]` compute-PSO отвергается. Лечится wave64-портом, а не native DXR.
- Предложение: расширить AGENTS п.8 с 5 до **7** репозиториев (`xcl` — только идеи, `XSEmu` — третья линия, GPL-3-файлы — с provenance), чтобы разрыв не повторился.
- Код не менялся; следующий шаг — по команде пользователя: port P0-1/P0-2/P0-3 (copy-back wait, saturating_sub, tiles A/B) отдельными коммитами с `cargo check/test` и парным A/B.

## §32. Раунд 9 боевой — ветка `feat-r9-async-copyback-and-water`: Step 0 (AGENTS 7 репо, нативные 2K, tiles 512), P0/P1 (`saturating_sub`, `ASYNC_COPY_BACK`, `EXACT_TEXBACK`, `DELTA_LAND`+tid), 6 портов из апстримов, release и 6 A/B-прогонов + smoke; **`DELTA_LAND` — измеренная регрессия**, остальные флаги opt-in; 12 коммитов, 665 passed

Ветка от `97ba4c0` (Round 9 research). `HEAD` на момент отчёта — см. `git log`; код менялся **только в `crates/victory-gpu` + `crates/victory-loader`**, `victory-shader` **не тронут** (14 файлов, ни одного в `victory-shader`), поэтому кэш `shaders-r8-group2` цел (76 973+ файла не перезаписывались). Коммиты: `743c03e` (Step 0), `ce8f626` (saturating_sub), `ae8c3c5` (`ASYNC_COPY_BACK`), `e9b3444` (`DELTA_LAND` + tid), `6320e4f` (`EXACT_TEXBACK`), `7d81ce0` (Belobug #414), `b86edd7` (Belobug #449), `aa504a1` (fxceless #1380), `633cfb9` (VItoriaBr #72), `5e1069e` (fxceless #1392), `f91b8ae` (Belobug #396).

### 32.1. Шаг 0 — правило 7 звёзд, нативные 2K, 512 МБ тайлов

- `AGENTS.md` п.8 расширен с 5 до **7** репозиториев (`xcl` — GPL-2.0, только идеи/факты; `XSEmu` — третья линия, GPL-3-файлы с provenance), коммит `743c03e`.
- `launch-game.ps1` / `run-road.ps1` (вне репо; снапшоты `audit\round9\harness-*-r9.ps1`, `harness-*-r9-final.ps1`): `-Force1080p 0` по умолчанию (возврат к нативным 2K; при 1080p интерфейс был огромным), `VICTORY_FPS_COUNTER=all` остаётся, `VICTORY_TILES_SMALL_WHEN_FULL_MB=512` (было 256, upstream-дефолт), `-Force1080p 1` — opt-in Round 8. Добавлены параметры `-AsyncCopyBack/-DeltaLand/-ExactTexback` (по умолчанию `''`, т.е. флаги выключены).

### 32.2. Шаг 1 — P0-1/P0-2

- **`saturating_sub` (`ce8f626`):** `shadow.rs::differing`, `at.saturating_sub((start + at) % PIECE).max(from)` (fxceless #1407, upstream `81d54b06f7`) + unit-тест `a_big_compare_finds_changes_in_its_first_slice` (48 МиБ, 3 потока, адрес на 32 КиБ внутри piece). Тест зелёный.
- **`VICTORY_ASYNC_COPY_BACK=1` (`ae8c3c5`):** при разборе выяснилось, что наш relay **уже включён по умолчанию** и для обычных чтений title-памяти ждёт только перекрывающиеся полёты (`relay_exact`, stage 2, `blocked_by(line.written, runs)`). Новый флаг расширяет это на **точку stale-refresh** (`wait_stale`): вместо глобального `settle(WHY_STALE)` ждём только полёты, которые могут писать освежаемые диапазоны (`blocked_by(..., refresh: true)`), затем повторно спрашиваем write-watch. Идея XCL_MAP_ASYNC (xcl #2952 п.5/#1993), off by default.
- A/B: `ab3` (on) против `ab4` (off) — 500–1500 draws 142.9 против 230.4 мс, top-thread 48 % против 70 %; но `ab6` (on) против `ab5` (off) — 271.7 против 217.3 мс. **Стабильного выигрыша нет**; флаг оставлен opt-in, дефолты не менялись.

### 32.3. Шаг 2 — writeback, дельта-лендинг, tid, порты

- **`VICTORY_EXACT_TEXBACK=1` (`6320e4f`):** в `texback.rs::land` readback пишет только изменившиеся run'ы (`shadow::changed_runs`) и пропускает неизменившиеся; в логах `texture readbacks skipped as unchanged (VICTORY_EXACT_TEXBACK)`. Это traffic-половина submission-authority writeback из XSEmu `image_writeback.rs` (родословная — наш PR #999); page-guard половина **не портирована**.
- **`VICTORY_DELTA_LAND=1` (`e9b3444`):** `relay_merge`/`card_merge_relay` пишут только изменившиеся run'ы вместо stretch от первого до последнего отличия. **Измеренная регрессия:** прогон `ab2` (все три флага) дал **~1 fps**, landing 526 мс/строку против 73–119 у остальных, top-thread 93 %. Причина: `delta_write` делает отдельный `write_mapped` на каждый run (десятки kernel-вызовов) вместо одного stretch; флаг оставлен **off**, переделку (батч по 64-КиБ piece или один merged write) — в следующий раунд.
- **tid потока-победителя (`e9b3444`, `stats.rs`):** load-строка теперь пишет `its busiest thread N % of one (victory-submit tid 14356)`; зарегистрированы `victory-submit/relay/heap/late-wait/wide/stats`. В `ab1` победитель — `victory-submit` (13/37 проб), затем `victory-heap` (4) и неназванные tid'ы; это подтверждает вывод coolfox-perf про submit-поток.
- **Порты (каждый отдельным коммитом, `cargo check`=0 после каждого):** Belobug **#414** `FRAME.load` до swap (`7d81ce0`); Belobug **#449** legal-state с обеих сторон host-barrier (`b86edd7`); fxceless **#1380** clzero на guarded/uncommitted линии через runtime-хендлеры, 151 строка (`aa504a1`); VItoriaBr **#72** placed-UAV aliases: мирроринг в host-heap, reuse target по адресу, alias в placed SRV (`633cfb9`, тесты адаптированы: `Held` derives, `alias_of(...).map(|h| h.get())`, `make(...).is_some()`); fxceless **#1392** partly-mapped never-filled текстура падает в `copy_mapped` вместо нулей (`5e1069e`, 3-way конфликт разрешён вручную, тест зелёный).
- **Belobug #396 портирован вручную (`f91b8ae`):** `Waiting.checked` — tuple `(u64,u64,u64,u8,u64)` → структура **`TextureRecheck`** (+`changed_in_place`), `recheck` принимает её целиком, `checked(index, write)` сверяет `write`; `HostHeap::texture_slot_recheck/checked` — новые сигнатуры; в `draw.rs` добавлены `next_texture_recheck()` (0→12, 1→40, 2→120, 3→400 кадров; текстура, однажды изменившаяся без уведомления, держит последний интервал 400 после конца начальных проверок), `changed_in_place` и `schedule_recheck`; тесты `only_previously_changed_textures_keep_the_final_recheck_interval` и `a_descriptor_rewrite_clears_periodic_texture_rechecks` зелёные. Это корректность незамеченных изменений текстур (не скорость); smoke 180 с после порта — чистый.

### 32.4. Шаг 3 — release и A/B-прогоны (нативные 2K, окно 1280×720 topmost, anti-AFK W/40 с, FPS=all)

`cargo build --release -p victory` = 35,27 с, `victory.exe` **9 285 120 Б** (до порта #396; после него release пересобран за 31,83 с → **9 286 656 Б**). Шесть прогонов:

| прогон | флаги | с | мир с | world n | world ms | fps | 500–1500 | 1500–3000 | top % | VRAM peak | ERROR |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `road\r9-ab1` | база | 260 | 72 | 99 | 241,0 | 4 | 200,8 | 204,3 | 57 | 7 276 | 0 |
| `road\r9-ab2` | все 3 | 260 | 89 | 34 | 1107,9 | 1 | 956,0 | 1436,3 | 93 | 7 124 | 0 |
| `road\r9-ab3` | async+exact | 280 | 77 | 109 | 167,9 | 6 | 142,9 | 186,3 | 48 | 7 332 | 0 |
| `road\r9-ab4` | база | 280 | 76 | 99 | 271,5 | 4 | 230,4 | 283,8 | 70 | 7 369 | 0 |
| `road\r9-ab5` | база | 280 | 58 | 142 | 210,4 | 5 | 217,3 | 210,4 | 56 | 7 504 | 0 |
| `road\r9-ab6` | async+exact | 280 | 68 | 103 | 337,6 | 3 | 271,7 | 340,4 | 62 | 7 445 | 0 |

- **Device loss/ERROR: 0 во всех шести прогонах**, VRAM peak 7 124–7 504 МиБ (< 8 192).
- Сцена между прогонами расходится сильно (world median draws 1 019–2 533, worldAt 58–89 с), поэтому **одиночная пара «флаги vs база» не доказывает выигрыш**: `ab3` заметно быстрее `ab4`, но `ab6` медленнее `ab5`; базовая дисперсия world-медианы 210–272 мс. Вывод: для решения нужен сцен-матчевый маршрут (фиксированная камера/сцена), а не свободный W-маршрут.
- **Визуально:** нативные 2K подтверждены (fpsbox читаемый, интерфейс не раздут; `ab1\shot-233` — пляж/лодки, артефактов от флагов не видно); `ab3\shot-236` — тёмная сцена с чёрным силуэтом и белыми точками (известная RT/свет-проблема, не привязана к флагу; встречается и в базовых прогонах). Плиты на воде в этих прогонах не выведены на сцену — отдельный boat-маршрут всё ещё нужен.
- **Smoke после порта #396** (`road\r9-bel396-smoke`, 180 с, база): world 200,5 мс медиана / p90 290,5, 11/11 шотов, 0 ERROR, 0 device loss, VRAM peak 7 559 МиБ, top-thread median 57 %, процесс завершился чисто (`exited cleanly`); в логе 206 срабатываний «changed after its upload with nothing to tell of it» — это и есть текстуры, которые теперь досматриваются на 400-м интервале.

### 32.5. Решение по дефолтам

Дефолты `launch-game.ps1`/`run-road.ps1` — **как в Step 0**: нативные 2K (`Force1080p=0`), `TILES_SMALL=512`, `FPS_COUNTER=all`; **ни один из трёх новых флагов не включён по умолчанию** (не подтверждены, `DELTA_LAND` — регрессия). Флаги остаются opt-in для следующего A/B. `victory-shader` и кэш шейдеров не тронуты.

### 32.6. Тесты и сборка

- `cargo check --workspace` = 0 после каждого шага/порта.
- Полный `cargo test --workspace -- --test-threads=1` зелёный, кроме известного warp-флейка §23.3 (`indirect::tests::warp_budget_saturates...`): он **воспроизводится и на чистом HEAD** (3/3, затем 4 попытки) — не связан с правками; по политике §23.3 финальные прогоны с `--skip` этого теста: **663 passed / 0 failed / 17 ignored** после портов и **665 passed / 0 failed / 17 ignored** после #396 (+7 тестов к Round 8). Логи: `audit\round9\r9-step1a-test*.log`, `r9-step1b-test*.log`, `r9-step2-ports-test.log`, `r9-bel396-test.log`.
- Новые тесты: `a_big_compare_finds_changes_in_its_first_slice`, `delta_landing_runs_are_the_stretches_that_differ`, `placed_unordered_texture_view_mirrors_and_aliases_to_shader_view`, `cached_placed_uav_is_visible_after_releasing_a_temporary_target`, `no_wait_unlocked_falls_back_for_never_filled_partly_mapped_texture`, `only_previously_changed_textures_keep_the_final_recheck_interval`, `a_descriptor_rewrite_clears_periodic_texture_rechecks`.

### 32.7. Артефакты

- Прогоны: `C:\GTA6_Port\road\r9-ab{1..6}\` и `road\r9-bel396-smoke\` (captions/vram/err/shot/env/version).
- Сводка: `audit\round9\ab-final-table.txt`, `ab1-summary.txt`, `ab2-summary.txt`; анализатор `audit\round9\ab-analyze.ps1`.
- Порты: `audit\round9\raw\ports\{fx1380,fx1392,vb72,bel414,bel449,bel396}.diff`; снапшоты харнесса `audit\round9\harness-*-r9*.ps1`.
- Тесты/сборка: `audit\round9\r9-step1a-test*.log`, `r9-step1b-test*.log`, `r9-step2-ports-test.log`, `r9-bel396-test.log`; release — `target\release\victory.exe`.

### 32.8. Ветка и следующий раунд

- Ветка **`feat-r9-async-copyback-and-water`** запушена в `myfork` (`4EKUSHKAq/victory-fork-3060ti`) после этого коммита; режим — Series X.
- Round 10: (1) `DELTA_LAND` переделать (батч изменённых run'ов в один `write_mapped`, замер landing); (2) сцен-матчевый A/B-маршрут (фиксированная камера, boat-сцена) для решения по `ASYNC_COPY_BACK`/`EXACT_TEXBACK`; (3) если флаги не дают стабильного выигрыша — вернуться к wave64 (3 refused CS) и VRAM-политике 75/85/92 % из xcl.

## 33. Раунд 10 — вода/RT-скипы, view-heap, native 1440p, AMBIENT_SPECULAR, DLSS-аудит

Дата: 27.11.2026. Ветка **`feat-r10-water-heapcap-and-1440p`** (от `feat-r9-async-copyback-and-water` @ `90c9da5`), режим Series X, окно 1280×720 topmost, сессия — 3060 Ti 8 GB. `victory-shader` **не тронут ни одним коммитом** (`git diff --name-only 90c9da5..HEAD` не содержит его), кэш `shaders-r8-group2` цел (25 641 файл, не переписывался).

### 33.1. Коммиты раунда

| коммит | что |
|---|---|
| `98d4a88` | docs(agents): радар Шага 0 — 6 NVIDIA-деревьев + обязательный `gh pr list` апстрима в каждом раунде |
| `a7fe4e8` | perf(gpu): water-light скип на 32-lane (5dc3b259…, 6f225b64…) + RT resolve/activate-rays без BVH (VIctory-NVIDIA 35c156087e) |
| `d7391d7` | perf(gpu): проба view-heap лимита при создании устройства (host-часть PR #3476; ядро клампа уже было в дереве) |
| `3b8c017` | feat(loader): `VICTORY_NATIVE_1440P` — SDR display mode + Settings viewport + temporal scale + output 2560×1440 (DexNom через VIctory-NVIDIA) |
| `71d3293` | feat(gpu): `VICTORY_AMBIENT_SPECULAR` — отражения без RT (VIctory-NVIDIA / foxhound, без ONCE) |
| (этот) | docs: HANDOFF §33 |

### 33.2. Шаг 0 — радар AGENTS.md

П. 8 дополнен: обязательный обход `trustno1-coder/VIctory-NVIDIA`, `readmet3xt-lab/foxhound`, `Steenax6/VIctory-NVID`, `Xeltax/VIcky`, `101Leafy/victory-leafy`, `Fayberr/Victory-United`; чужой exe не запускать (только исходники/диффы); `gh pr list -R Marvisak/VIctory --state open --limit 100` в каждом раунде; provenance-нота в коммите каждого порта.

### 33.3. Шаг 1 — вода/RT-скипы и PR-решения

- **Вода/RT (из `VIctory-NVIDIA` 35c156087e).** На 32-lane хосте `CS_TransmissionVolumetrics_Tiled` (5dc3b25986622949) и `_Tiled_LocalLights` (6f225b641c5b0708) идут только в wave-wide форме и оставляют тёмные плиты открытой воды; без BVH `CS_RaytracingResolve`/`CS_ScreenProbes_ActivateRays` шейдят несуществующие попадания. В нашем дереве это `pipeline::left_out` (считается при создании `Pending`, до первого перевода; закрывает `make::ready/early/now` и попадает в `missing()` для диагностики): `VICTORY_WATER_LIGHT=1`, `VICTORY_RT_RESOLVE=1`, `VICTORY_ACTIVATE_RAYS=1` возвращают. Ведущее 32-lane хранится в `victory-gpu::host::host_max_wave()` (своя проба `WaveLaneCountMax`), `victory-shader` не тронут. Два unit-теста на решение.
  **Подтверждено прогоном r10-run1:** в `err.log` три строки «a pipeline is left out for the whole run» — RT resolve `7f05ddab771799a9`, activate rays `39f0e9aeb156a61c`, вода `5dc3b25986622949`; в r10-run2/Run3 — второй водный `6f225b641c5b0708`. Побочно видны и «host pipeline refused 0x80070057» для части wave64-CS (2da4beb3, 02384a6, 6b82bc7d) — это отказ D3D12 на PSO, не наш скип; лечится только wave32-понижением (отдельный раунд).
- **PR #3476 (view-heap clamp):** ядро уже в дереве (`set_view_heap_limit`, `probe_view_limit`, `translation_fingerprint` с лимитом; линия `fix/nvidia-view-heap-cap`/`fix/amd-view-heap-clamp`), прогон подтвердил: «host view heaps hold 1000000 views; view indices are clamped to the null slot». Из PR взята его host-часть — проба лимита **при создании устройства** (`probe_view_limit_on(&device)`), чтобы ни одна трансляция/ключ кэша не увидели лимит=0 (`d7391d7`). Переименования в victory-shader не переносились (ломают кэш, выгоды нет).
- **PR #3465 (NVIDIA AV):** не портирован — это **одна строка в тесте** `victory-shader/src/emit/memory_tests.rs`, которого в нашем дереве нет, а любая правка `.rs` под `victory-shader/src` меняет `VICTORY_SHADER_SOURCE_HASH` (build.rs хэширует всё) и убивает кэш. Production-кода PR не меняет.
- **PR #3378 (очередь пайплайнов):** не портирован — архитектурно другая очередь (`queued/now/first/taken` + 2 очереди + `shown.rs`/`feeds.rs`, ~1 050 строк по 9 файлам), с нашим `make.rs` (один `queue_state: AtomicU8`, один `QUEUE`) чисто не совместим; кандидат отдельного раунда вместе с #3396/#3419.
- Полный разбор решений: `audit\round10\r10-pr-decisions.md`, диффы — `audit\round10\raw\prs\`.

### 33.4. Шаг 2 — AMBIENT_SPECULAR и native 1440p

- **`VICTORY_AMBIENT_SPECULAR=<share>[,<sky>]`** (`71d3293`, порт субагентом): `Role::Miss` (`CS_RayMiss_<8hex>_`, не `_Volumetrics` и не ScreenProbe), проход `reflect_light` (HLSL) сразу после miss-программы и до `CS_ResolveScreenSpace`: оценка света вдоль отражённого направления из тех же cells/points; `words[6/7] = depth_size` (картинка отражений половинная), fallback-сканы мест бинда (обычное место (1,104)/(0,1000), затем UAV 2D f26 и depth-like 32-bit ~2× размера, stencil-плоскости отбраковываются, `placed_texture` для placed-записей), `VICTORY_AMBIENT_SPECULAR_ALPHA=1` — alpha слой; `on()` = (AMBIENT задан ИЛИ SPECULAR задан) И builder off; info-строка раз в 10 с. `VICTORY_AMBIENT_ONCE` не переносили (раскладка cells осталась `2*CELLS+1`, без SHARES). Тесты: `specular_settings`, miss-имена в `roles_by_name`, `the_passes_build` реально компилирует новый проход (0,60 с на D3D12).
  **Прогон r10-run1:** 32 info-строки; последняя — «45 frames, 62 dispatches of CS_RayMiss followed (surfaces: opaque), … 124 of water or the alpha layer and 0 before a frame's light was kept» → layer определяется верно (opaque), water/alpha честно пропускаются. Полной проверки «красиво/нет» нет: в кадрах вода местами тёмная с пурпурными полосами (`r10-run1\shot-187`), а сцена пляжа пересвечена в белое (`shot-248`) — это кандидат на тюнинг (`share 0.6, sky 0.25`, `AMBIENT 0.6/AIR 1.2`), а не доказанный дефект порта.
- **`VICTORY_NATIVE_1440P`** (`3b8c017`): четыре поля титула, каждое сверено побайтно: SDR display mode `+0x615c100` (3840×2160@60 → 2560×1440), Settings viewport load `+0x58048d4` (переадресован на SDR mode), temporal scale `+0x28f1e7e` (2/3 → 1.0), output 2560×1440 через наш `OUTPUT`; pool-check перепрыгнут. Скан-аут резервация `+0x247ad4a` (64 MB) **оставлена** — наш pool-check всё равно перепрыгнут; экономия ~34 MB — follow-up, нужен `victory-formats` размер текстуры.
  Сверка с образом: **BLAKE3 образа = b99f876b74d4a77f03af5f878223b27bd647b5c35a79d5decd9f716504239e93 — ровно retail build, который называет NVIDIA** (SHA-256 файла e95020b2… — другая величина, это и было «расхождение»); PE-разбор `audit\round10\pecheck.py` подтвердил stock-байты на всех их RVA и на всех наших старых сайтах. Во всех трёх прогонах лог: «resolution: patched … Done» для 5 сайтов (0x224352b, 0x247af25, 0x615c100, 0x58048d4, 0x28f1e78).

### 33.5. Шаг 3 — DLSS: read-only аудит

Отчёт `C:\GTA6_Port\audit\dlss-prep.md` (206 строк). Кратко: нужны Streamline SDK **v2.14.16** + 8 DLL (`sl.interposer`, `sl.common`, `sl.dlss`, `sl.dlss_g`, `sl.reflex`, `sl.pcl`, `nvngx_dlss`, `nvngx_dlssg`), качаются пользователем (в git не кладём; лицензии рядом); перехват — подмена **экспортов IAT** `amd_fidelityfx_loader_xbox.dll` (`ffxCreateContext/ffxDestroyContext/ffxDispatch/ffxQuery`), DLSS-ветка в `streamline::upscale()`, `slEvaluateFeature` пишется в наш host-список; **FG на 3060 Ti недоступен** (RTX 40+), только SR. У нас нет трёх кусков: патча рендерера 7 (FidelityFX-путь), перехвата `ffxDispatch`, модуля Streamline; подробный 8-шаговый план следующего раунда — в отчёте. Ожидание по FPS — единицы процентов (кадр ~200 мс при GPU ~33 мс), ценность — качество апскейла; VRAM: пик уже 7,1–7,5 GB, запас < 1 GB.

### 33.6. Шаг 4 — сборка и три прогона в 2K

`cargo build --release -p victory` = 46 с, `target\release\victory.exe` 9 301 504 Б (SHA-256 47755A9F…; после прогона №3 восстановлен из `audit\round10\victory-release-r10.exe`). Для прогона №3 собран диажный профиль: `cargo build --profile diag -p victory --features diagnostics` (11 129 344 Б, DRED доступен только в diagnostics).

Харнесс: anti-AFK теперь **поворот камеры `L` 120 мс каждые 40 с** (правый стик, gameinput.rs:134-135) + `W` 240 мс каждые 120 с (продвижение маршрута). Новые дефолты: `AMBIENT=0.8`, `AMBIENT_AIR=1.5`, `AMBIENT_SPECULAR=1`, `NATIVE_1440P=1` (перекрывает `FORCE_1080P`), `FPS_COUNTER=all`, `TILES_SMALL=512`; снапшоты `audit\round10\harness-launch-game-r10.ps1`, `harness-run-road-r10.ps1`.

| прогон | флаги | с | worldAt | world n | world median | p90 | max | последний кадр | VRAM peak | device loss / ERROR |
|---|---|---|---|---|---|---|---|---|---|---|
| `road\r10-run1` | specular=1, native1440p | 420 | 88 | 179 | 216,2 | 405,7 | 4 087,7 | 6 fps / 166,2 мс / 2625 draws | 7 461 | 0 / 0 |
| `road\r10-run2` | control: specular=0, native1440p | 240 | 88 | 53 | 182,6 | 509,1 | 2 963,6 | 6 fps / 171,5 мс / 1676 draws | 7 152 | 0 / 0 |
| `road\r10-run3` | RT diag: `BVH_BUILDER=1 BVH_QUARANTINE=1 DRED=1`, ambient/specular=0, **diag-бинарник** | 240 | — | — | — | — | — | до мира не дошёл (6 шотов меню) | 3 393 | 0 / 0 (CPU AV) |

- **Run 1** дошёл до лодочной/островной сцены (shot-187 — лодка и вода, shot-310 — палуба с персонажами, shot-403 — финал), финальная фаза 6 fps / 166 мс. Плиты на воде **не выводятся однозначно**: вода тёмная с пурпурными полосами, часть кадра пересвечена (см. 33.4).
- **Run 2 (контроль)** — та же сцена-окрестность, specular-строк 0; сравнение 88–240 с: run1 213,3 мс / 1 449 draws против run2 182,6 мс / 1 117 draws; в пересчёте на draw 0,147 против 0,163 мс — **измеримой цены SPECULAR не видно**, разброс сцены больше эффекта (как и в §32.4). Вывод: флаг оставляем включённым по умолчанию в харнессе, но «доказанным» не называем.
- **Run 3 (RT-диагностика):** RT-билдер **до мира не доходит**. Титул падает в загрузке на ~125 с: `first-chance/unhandled exception 0xc0000005 at victory-image-b99f876b74d4a77f.dll+0x23cf683 (write 0x0: nothing mapped there)`, далее BacktraceBreakpad/minidump (HTTP заблокирован). Device removal/DRED-событий нет (это CPU-AV, не потеря устройства), VRAM peak 3 393 MB, builder-диспетчей в логе нет. **RT остаётся выключенным по умолчанию**; для разбора нужен отдельный раунд (адрес 0x23cf683, QUARANTINE-путь HLE).
- Во всех прогонах `0 ERROR`, `0 device loss`; медленные кадры 2–4 с — это собственные паузы титула (preparing/копии/подгрузка), не флаги (видны и в контроле run2).

### 33.7. Тесты и сборка

- После Шага 1 (a+b): полный `cargo test --workspace -- --test-threads=1` упал **только** на известном warp-флейке §23.3 (`indirect::tests::warp_budget_saturates…`); ретрай флейка — снова FAIL (ожидаемо), финальный гейт с `--skip` этого теста: **667 passed / 0 failed / 17 ignored** (+2 новых теста Шага 1).
- После Шага 2 (SPECULAR субагентом): **669 passed / 0 failed / 17 ignored** (`audit\round10\test-step2-specular.log`), `the_passes_build` компилирует новый REFLECT-проход.
- `cargo check --workspace` — 0 после каждого шага. Финальное дерево побайтно совпадает с протестированным (`git diff 77e17e0 HEAD` пуст после починки сплита коммитов).
- Логи: `audit\round10\test-step1a.log`, `test-step1a-skip.log`, `test-step1b.log`, `test-step2-specular.log`, `build-release.log`, `build-diag.log`.

### 33.8. Артефакты

- Прогоны: `C:\GTA6_Port\road\r10-run{1,2,3}\` (captions/vram/err/out/shot/env/version/keys/watch).
- Анализ: `audit\round10\r10-analyze.ps1`, `analyze-run{1,2,3}.log`, консоли `run{1,2,3}-console.log`.
- Порты/решения: `audit\round10\r10-pr-decisions.md`, `raw\prs\` (метаданные и диффы #3476/#3465/#3378), `raw\ambient-specular.diff`, `pecheck.py`, `victory-release-r10.exe`.
- DLSS: `audit\dlss-prep.md`. Харнесс: `audit\round10\harness-*-r10.ps1`.

### 33.9. Ветка и следующий раунд

- Ветка **`feat-r10-water-heapcap-and-1440p`** запушена в `myfork` (`4EKUSHKAq/victory-fork-3060ti`); локально = remote; `victory-shader` и кэш шейдеров целы.
- Round 11 (по приоритету): (1) визуальный тюнинг `AMBIENT_SPECULAR` (0.6/0.25, затем `AMBIENT_SPECULAR_ALPHA=1` на воде) и `AMBIENT 0.8→0.6` — убрать пересвет/пурпур; (2) `native_1440p` скан-аут резервация (−34 MB) через `victory-formats` размер текстуры; (3) DLSS SR по плану `audit\dlss-prep.md` (без правок victory-shader) — сначала `ffxDispatch`-перехват и `sl.interposer`, FG не трогать; (4) разбор RT-краша `0x23cf683` (BVH+QUARANTINE, diag-бинарник); (5) сцен-матчевый A/B-маршрут (фиксированная камера/boat-сцена) для specular/ASYNC/EXACT; (6) кандидаты PR #3378/#3396/#3419 и wave64-понижение (3 refused CS).
