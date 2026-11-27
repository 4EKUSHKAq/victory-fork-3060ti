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

## 34. Раунд 11 — тихий харнесс, настоящий 1080p с нормальным UI, скан-аут −46 MB, FidelityFX-путь и задел под DLSS (2026-11-27)

Ветка **`feat-r11-1080p-ui-dlss-and-quiet`** от `feat-r10-water-heapcap-and-1440p` @ **`540d9c9`**. Коммиты: **`6287417`** (HANDOFF §1–§28 → `HANDOFF-archive.md`), **`d4c8cd8`** (AGENTS: тихий харнесс, DLSS/Streamline-правила), **`e957bf6`** (Шаг 1: `VICTORY_NATIVE_1080P` + скан-аут), **`05e7e2b`** (VICTORY_KEYS в релизе), **`340af27`** (Шаг 2: FidelityFX-путь + перехват ffx-импортов), **docs-коммит** (§34, сразу после `340af27`). `victory-shader` — **0 файлов** (`git diff --name-only 540d9c9..HEAD -- crates/victory-shader` пуст), кэш `shaders-r8-group2` не тронут.

### 34.1. Шаг 0: сжатие контекста, радар, тихий харнесс

- `HANDOFF.md` сокращён с 2570 до ~390 строк: §1–§28 (раунды 1–6, окружение, старые команды) перенесены в `HANDOFF-archive.md`; в основном файле шапка + §29+.
- Радар (один read-only субагент, отчёт — `audit/round11/r11-radar.md`): апстрим `Marvisak/VIctory` (50 открытых PR), `fxceless/victory-fork` (HEAD `9a9091d180`), NVIDIA-линия. Главные факты: **(1)** «1080p-output patch» от hyphennn **не существует** — «hyphennn» это прежнее имя/сообщество форка `fxceless/victory-fork`, а его README/ветки/PR не содержат такого патча; наш рецепт (DexNom/trustno1 `9a791323ba`/`d11233aa22` при 1920×1080) — единственный проверенный. **(2)** DLSS-стек trustno1 (`63957271cd`) полностью есть в уже запинованном дереве `c437b0594b` (fidelityfx.rs 508, streamline.rs 1978, abi 679, plan 515, external.rs 51, native_1440p.rs 214 строк) — источник для следующих раундов. **(3)** В апстриме появились полезные для 8 ГБ вещи: 101Leafy #26 (RT-резерв 3072 MB и до 4 ГиБ spares не отмасштабированы под 24 ГиБ), merged #2286/#21/#2234, trustno1 `47747d6236` (дескрипторные слоты) и `826ba150c8` (фикс фиолетового мусора ambient-проб), `e31f474068` (VICTORY_SKIP_BINK), Sr-0w `bcdb9730e4` (VICTORY_KEYS в релизе — взят).
- Тихий харнесс (`run-road.ps1`): **убраны** `HWND_TOPMOST`/`SetWindowPos(HWND_TOPMOST)`, `sendkeys2.ps1`, `AttachThreadInput`, `SetForegroundWindow`, `keybd_event`, Alt+F4 и весь расписанный ввод; окно — обычное `1280×720` в нормальном z-порядке (`HWND_NOTOPMOST`, `SWP_NOACTIVATE`), скриншоты без фокуса (`shot-<t>.png`), конец — `CloseMainWindow` → kill. `launch-game.ps1` тоже больше не делает окно topmost (фокус один раз для интерактивной игры). Путь к миру проходит через **VICTORY_KEYS** (клавиши инжектируются в HLE, `gameinput::scripted`, теперь в релизных сборках — порт Sr-0w `bcdb9730e4`).
- Свет по умолчанию в обоих скриптах: `AMBIENT=0.6`, `AMBIENT_AIR=1.2`, `AMBIENT_SPECULAR=0.6,0.25`, `AMBIENT_SHADE=0.3,0.4`, `FPS_COUNTER=all`, `TILES_SMALL=512`. AGENTS п.9 (тихий харнесс) и п.10 (DLSS/Streamline, DLL не коммитить) добавлены; §1–§28 — в архиве.

### 34.2. Шаг 1: настоящий 1920×1080 с нормальным масштабом UI + скан-аут (коммит `e957bf6`)

`crates/victory-loader/src/force_1080p.rs` (+121/−12), `victory-formats` в зависимости загрузчика:
- `VICTORY_NATIVE_1080P=1` (важнее `NATIVE_1440P`): родные 1920×1080 в **запись SDR-режима** (+0x615c100 → те же 16 байт, что у собственной 1080p-строки игры +0x615c0e0: `1920×1080@60 Гц, флаг 1`), `Settings viewport` (+0x58048d4) читает её же, `OUTPUT` (+0x224352b) = 1920×1080, обход проверки пула (+0x247af25), render scale 1.0 (+0x28f1e78). Раньше (Раунд 8) менялся только размер вывода, а UI оставался 4K — отсюда «раздутый интерфейс»; теперь UI разложен под 1080p.
- **Скан-аут резервация** (+0x247ad4a, `mov ecx, 4000000h` = 64 MB под две 4K-плитки) пересчитывается через `victory_formats::texture::Texture` (RGBA8, `SW_64KB_D_X`, пара буферов, округление до 2 MiB): **1080p = 18 MB (экономия 46 MB)**, 1440p = 30 MB (34 MB). Проверено на байтах stock: 4K-пары = ровно 64 MB, значит формула воспроизводит оригинал.
- Тест `the_native_1080p_fields_are_the_titles_own_1080p_mode` (сравнение с реальной 1080p-строкой игры, размеры пула, код скан-аута).
- Гейт: `cargo check --workspace` — 0; `cargo test --workspace -- --test-threads=1` — **669 passed / 0 failed / 17 ignored** (известный warp-флейк §23.3 воспроизвёлся и при ретрае; финальный гейт с `--skip warp_budget_saturates_without_wraparound_and_preserves_draw_fields`).

### 34.3. Шаг 2: DLSS-подготовка — Streamline SDK, FidelityFX-путь, перехват ffx (коммиты `05e7e2b`, `340af27`)

- **SDK скачан и проверен**: `C:\GTA6_Port\scripts\get-nvidia-sdks.ps1` (порт NVIDIA-скрипта, GPL-3.0) → `C:\GTA6_Port\streamline\bin\x64`: 8 DLL (~70 MB, sl 2.14.16, nvngx 310.9.16), все с **валидной Authenticode-подписью NVIDIA Corporation** (`check-streamline.ps1`, лог `audit/round11/check-streamline.log`). В репозиторий не коммитятся; `VICTORY_STREAMLINE_DIR=C:\GTA6_Port\streamline\bin\x64` прописан в `launch-game.ps1` и `run-road.ps1`.
- **Патч рендерера 7** — `crates/victory-loader/src/upscaler.rs` (порт `prepare_upscaler` DexNom/VIctory через trustno1): `VICTORY_UPSCALER=7` / `VICTORY_DLSS=<mode|1>` (dlaa/quality/balanced/performance/ultraperformance, `1`=quality): `RENDERER_SELECTION +0x7e9421` → 7, `UPSCALER_SIZING +0x28ef69a` → режим FFX, `NATIVE_LOAD +0x28ef6a8` / `SCALED_LOAD +0x28ef6b2` → масштаб режима, `UPSCALER_SCALE +0x615d95c` (2/3) общий — не пишется; temporal scale (render scale) = масштабу режима через `asked_render_scale`.
- **Перехват ffx-импортов** — `crates/victory-hle/src/modules/fidelityfx.rs` (порт `Interpose` trustno1): 5 обёрток (`ffxCreateContext/DestroyContext/Configure/Query/Dispatch`) подменяют IAT-экспорты `amd_fidelityfx_loader_xbox.dll`, зовут оригинал и **читают** upscale-dispatch (`kind 0x10001`): логируются контекст (`max_render/max_output`) и первый dispatch (render/upscale/jitter/MV). **DLSS SR в этом раунде не исполняется**: обёртка возвращает dispatch заголовку (см. 34.5) — это точка подключения Streamline `slEvaluateFeature` (план `audit/dlss-prep.md`, шаги 5–6).
- Гейт: **674 passed / 0 failed / 17 ignored** (те же `--skip`), `check` — 0.

### 34.4. Шаг 3: релиз и три тихих фоновых прогона (2K-окно `1280×720`, без topmost и клавиатуры)

Сборка: `cargo build --release -p victory` — 37,7 с, exe **9 325 568 Б**. Окно `1280×720` обычное; ввод — только `VICTORY_KEYS` внутри HLE; `AMBIENT=0.6/AIR=1.2/SPECULAR=0.6,0.25`; телеметрия `all`; 0 device loss / 0 ERROR во всех прогонах; захвачены скриншоты.

| # | Прогон | Что включено | worldAt | world n | median | p90 | draws med | VRAM peak | итог |
|---|--------|--------------|---------|---------|--------|-----|-----------|-----------|------|
| 1 | `road\r11-run1` 300 с | native 1080p, render 1.0, `DLSS=0` | 67 | 237 | **159,6 мс (6,27 fps)** | 749,0 | 1 412 | **6 655 MB** | картинка нормальная, UI нормальный, все 6 сайтов + скан-аут Done |
| 2 | `road\r11-run2` 300 с | native 1080p, FidelityFX-путь (`DLSS=quality` → 1280×720→1920×1080) | 65 | 242 | 102,8 мс | 360,0 | 1 072 | 5 980 MB | **3D-картинка чёрная**: FSR4-пайплайны не встали |
| 3 | `road\r11-run3` 240 с | native 1080p, `RENDER_SCALE=0.6667` (рендерер 6, temporal upscale 1280×720→1080p) | 85 | 163 | **113,6 мс (8,80 fps)** | 235,5 | 1 494 | **6 079 MB** | картинка нормальная |

Сравнение по равным корзинам `draws` (Run1 vs Run3, корзина 100): при 1300–1700 draws Run3 быстрее на **−1…−67 мс** (медиана дельт ≈ −21 мс), VRAM пик −576 MB (6 655 → 6 079). Сцены и маршруты разные (камеры/погода), поэтому это индикатор, а не доказанный A/B: сцену нужно матчить (следующий раунд). Run2 (чёрный кадр) по времени не сравнивать.

### 34.5. Что подтвердилось и что нет

**Подтвердилось (логи `out.log`/`err.log`):**
- native 1080p: `resolution: settings' output size applied (1920x1080)`, `SDR display mode (3840x2160) (1920x1080)`, `Settings' preview viewport size load (1920x1080)`, `scanout reservation (64 MB) (1920x1080)`, `render scale (1)` — все **Done**; в run2/render scale — `0.6666667`.
- тихий прогон доходит до мира **без единого физического нажатия**: `world (500+ draws) at 67` (run1), до этого — только `VICTORY_KEYS`-скрипт в HLE; окно не topmost и не забирает фокус.
- DLSS-плумбинг точный: interpose привязал `ffxCreateContext`; контекст `max_render=[1280,720] max_output=[1920,1080]`; первый dispatch `render=[1280,720] upscale=[1920,1080] jitter=[0,0] motion_scale=[1,1]` — ровно размеры DLSS Quality при 1080p-выходе.
- скипы из Раунда 10 живы: water-light `5dc3b259…` и RT-пайплайны left out; view heaps 1 000 000; `CS_RayMiss`-строки (24 в run1).

**Не подтвердилось / осталось:**
- **DLSS SR не работает**: Streamline-движка нет (наша обёртка отдаёт dispatch заголовку), а собственный FSR заголовка даёт **чёрный 3D-кадр**: `FFX FSR4 ACO: DeserializeComputePipelineStateX FAILED hr=0x80004001` + host pipeline refused `0x80070057` для wave64-compute (`wave_lanes=[64,64]`). То есть для DLSS-профиля нужен либо `slEvaluateFeature`-движок (шаги 5–6 плана: host-копии/alias/upload/depth twin + запись в host-list; в нашем дереве нет `external.rs`/`HostList` NVIDIA-дерева — ~2,2 тыс. строк адаптации), либо понижение wave64 в `victory-shader` (заморожен в этом раунде). `VICTORY_DLSS=0` — гарантированный откат.
- **скан-аут −46 MB** подтверждён кодом/тестом, но отдельного A/B «без патча скан-аута» не делали (VRAM-разница между прогонами объясняется в основном 1080p-буферами и сценой).
- **UI**: нормальный размер виден по субтитрам/надписям (шоты run1 shot-90/285, run3 shot-150); Settings-меню со скриншотом не поймали — проверку «интерфейс не раздут» стоит повторить на статичном меню.
- **пересвет пляжа** (белый песок/розовое небо, run1 shot-285) и **тёмные лица** (run1 shot-150/225) остались: это не ambient-порог, а недостающий свет/тени RT (известное ограничение) — нужен отдельный раунд визуала.
- `AMBIENT_SPECULAR=0.6,0.25` применён (24–25 info-строк, «44 dispatches of CS_RayMiss»), визуальной разницы с 1.0 по одному кадру не оценить — нужен сцен-матч.

### 34.6. Артефакты

- Замеры: `C:\GTA6_Port\road\r11-run{1,2,3}\` (captions/vram/err/out/shot/watch/env/version).
- Анализ: `audit\round11\r11-analyze.ps1`, `r11-compare.ps1`, `analyze-run{1,2}.log`, `compare-runs.log`, `compare-run1-run3.log`, `run{1,2,3}-console.log`, `test-step1*.log`, `test-step2.log`, `build-release.log`.
- Радар: `audit\round11\r11-radar.md`; SDK: `C:\GTA6_Port\scripts\get-nvidia-sdks.ps1`, `check-streamline.ps1`, `audit\round11\check-streamline.log`; Streamline: `C:\GTA6_Port\streamline\` (вне репозитория).
- Харнесс: снапшоты `audit\round11\harness-*-r11.ps1`; `audit\dlss-prep.md` — план DLSS.

### 34.7. Следующий раунд (предложение)

1. **Streamline SR** (шаги 5–6 плана `audit\dlss-prep.md`): `slInit`/`slSetD3DDevice`/`slDLSSGetOptimalSettings`/`slSetConstants`/`slSetTag`/`slEvaluateFeature` в наш host-list + подготовка входов (alias/upload/depth twin) — прототип на нашем `queue.rs`/`list.rs`; тогда `VICTORY_DLSS=quality` реально исполнится. До этого — VRAM-запас: 101Leafy #26 (RT-резерв 3072 MB → по бюджету карты, spares 4 ГиБ) и upstream #3447.
2. FSR4-пайплайны: либо wave64-понижение (потребует `victory-shader` и пере-тёплой cache — отдельный раунд), либо дождаться DLSS-движка и ходить сразу через него.
3. Сцен-матч A/B (фиксированная камера/сцена) для native 1080p против 720p+temporal upscale и для `AMBIENT_SPECULAR`; снять Settings-меню.
4. Визуал: пересвет пляжа/тёмные лица/фиолетовые блики (`826ba150c8` ambient probe garbage — дешёвый кандидат).
5. Мелочи: `e31f474068` `VICTORY_SKIP_BINK` (быстрее холодные прогоны), `47747d6236` дескрипторные слоты, 101Leafy #26.

## 35. Раунд 12 (в работе) — диагностика кадра, масштаб UI, фиолетовый мусор проб, план lit-фолбэка теней

Ветка **`feat-r12-visual-light-fixes`** от `feat-r11-1080p-ui-dlss-and-quiet` @ **`6d12af6`**. Коммиты: **`ff09430`** (Шаг A: диагностика в `capture.rs`), **`b77a41e`** (B1: фиолетовый мусор проб), этот docs-коммит. `victory-shader` не тронут.

### 35.1. Что изучено (внешний документ)

Пользователь прислал `C:\Users\4EKUSHKA\Downloads\plan-rendering-parallel-discord.md` — это план проекта **`ubiquitarianism/leonida`** (C++/Vulkan-слой для GTA VI на Linux/RADV, тот же образ 1.0.2.1), не VIctory: прямых патчей нет, опыт AMD-only. Полезное: диагностика экспозиции через 1×1 luminance (у нас `capture.rs` уже логирует, но **не считал non-finite**), «чёрные лица» могут быть из-за недогруженных входов decoration bake, а не только RT; **тени — это `R32_UINT` shadow-mask таблица** (в нашем `capture.rs:697-720` уже описан её формат: 0 = полностью в тени, 1..255 = значение блока, >255 = atlas-блок, `0xFFFFFFFF` = никогда не записан), а не R9G9B9E5 light image; RT на их числах требует 24 ГБ и у нас заблокирован. Их «slot-133 shadows» — не маски теней (проверено по их docs).

### 35.2. Шаг A — диагностика без изменения поведения (`ff09430`)

- `crates/victory-gpu/src/capture.rs`: в лог `target values` добавлен `not_finite` (NaN/Inf в целях кадра, включая 1×1 exposure/luminance — раньше фильтровался из low/high), в `target texel` — счётчик non-finite каналов; в дампе `R32_UINT`-таблиц — доли `zero_percent` и `all_ones_percent` (доля «полностью в тени» и «никогда не записан»).
- `audit/round12/r12-analyze.ps1`: региональные метрики по скриншотам (mean/median luma, dark-share по прямоугольникам лицо/вода/пляж/небо, fpsbox исключён; регионы задаются долями кадра `-Regions`), плюс captions (world median/p90) и VRAM peak.
- Гейт: `cargo check --workspace` 0; `cargo test --workspace -- --test-threads=1 --skip warp_budget_…` — 0 failed (известный флейк §23.3 пропущен).

### 35.3. B0 — масштаб интерфейса: измерено и решено

Причина «огромного» UI: интерфейс раскладывается в **абсолютных пикселях вывода**, а не в долях кадра. Замер по меню (одно окно `1280×720`, скриншоты `road\r12-ui1080\shot-30.png` и `road\r12-ui1440\shot-30.png`):
- `VICTORY_NATIVE_1080P=1` (вывод 1920×1080): «ВЫБОР УЧЁТНОЙ ЗАПИСИ» ≈ **355×38 px**, глифы Y/A ≈ 30 px;
- `VICTORY_NATIVE_1440P=1` + `VICTORY_RENDER_SCALE=0.5` (вывод 2560×1440, сцена 1280×720): та же надпись ≈ **246×30 px**, т.е. **на ~30 % меньше** — нормальный масштаб 2K-интерфейса при рендер-косте ~720p.
Решение (закреплено в `run-road.ps1` и `launch-game.ps1`): **по умолчанию `Native1440p=1` + новая ручка `-RenderScale 0.5`** (`VICTORY_RENDER_SCALE=0.5`); `Native1080p` остаётся опцией. Отдельного «HUD/Scaleform-сайта» под 1080p не нашли: судя по абсолютным пикселям, его и нет — вернуть 1080p с нормальным UI можно лишь патчем UI-scale, а не размера дисплея (кандидат для следующего раунда, если он нужен). Прочее из профиля Раунда 11 сохранено: `VICTORY_DLSS=0`, `VICTORY_FPS_COUNTER=all`, `TILES_SMALL=512`, тихий харнесс без topmost/физического ввода (`VICTORY_KEYS`).

### 35.4. B1 — фиолетовый мусор ambient-проб (`b77a41e`)

Порт trustno1-coder/VIctory-NVIDIA `826ba150c8` (GPL-3.0-or-later): 9 строк HLSL в наш `ambient.rs::raise` — перед подъёмом света сбрасываются magenta/violet остатки в light image (зелёный < 0.5×min(красный, синий) и > 0.02; и синий > 2.5×зелёного при синем > красного). Возможен второй источник фиолетового — наш reflection-стенд-ин (проверяется в B4 A/B `SPECULAR=0/0.2/0.6`). Гейт: check 0, test 0 failed.

### 35.5. Что осталось (следующий сеанс)

- **B2:** lit-фолбэк `R32_UINT` shadow-mask таблицы. Уже есть инструменты: `capture.rs` пишет классы и доли. Нужен диагностический кадр с **diag-бинарём** (`VICTORY_CAPTURE`, feature `diagnostics`) — у него отдельный ключ кэша шейдеров, потребуется прогрев; затем найти ресурс маски (по формату 42 и классам: много `0`/`0xFFFFFFFF` при выключенном RT) и ставить мягкий lit-уровень (255) для незаписанных/полностью теневых блоков, opt-in флагом (`VICTORY_RT_SHADOW=lit`) и дефолтом после A/B.
- **B3:** по дампу проверить decoration bake (недогруженные входы при чёрных лицах) — `VICTORY_BAKE_SOURCES=1` уже включён, но нужен upload-ahead, если дамп это покажет; плюс запас 8 ГБ (101Leafy #26, `BAKE_TEXTURES_MS=25`).
- **B4:** тихий сцен-матч A/B (`RENDER_SCALE=0.5`, свет 0.4–0.6, `SPECULAR=0/0.2/0.6`, `SPECULAR_ALPHA=1`) с региональными метриками `r12-analyze.ps1`, закрепление лучших значений, затем §35-итог и push.

### 35.6. Артефакты

`audit\round12\`: `check-stepA.log`, `test-stepA.log`, `check-b1.log`, `test-b1.log`, `build-release.log`, `r12-analyze.ps1`, `harness-*-r12.ps1`, `ui1080-console.log`, `ui1440-console.log`. Скриншоты сравнения UI: `C:\GTA6_Port\road\r12-ui1080\`, `C:\GTA6_Port\road\r12-ui1440\`. Документ-источник разбора: `C:\Users\4EKUSHKA\Downloads\plan-rendering-parallel-discord.md`.

### 35.7. B2–B4 (продолжение) — изоляция провала DELTA_LAND, батчевая запись, закрепление дефолтов

**Изолированный A/B (без `AMBIENT_SPECULAR_ALPHA`), сцена «Люси сажают в машину копов», 243–299 с:**
- `r12-run1` (`DELTA_LAND=0`): world median **161,4 мс**, p90 386,7, max 1049,1; copy-back 18–108 мс;
- `r12-delta1` (`DELTA_LAND=1`, старый код): median **1251,4 мс**, p90 2451,1, copy-back до **1751 мс**;
- `r12-run2` (`DELTA_LAND=1` + `ALPHA=1`): median 1167,1 — как delta1, значит **`AMBIENT_SPECULAR_ALPHA` ни при чём**, провал даёт именно `DELTA_LAND` (мелкие `write_mapped` в watch-память на каждом кадре шторма).

**B3 (частично):**
- `crates/victory-gpu/src/shadow.rs`: батчевый delta-land — близкие run'ы (разрыв ≤ **512 Б**) склеиваются в один `write_mapped`, при > **64** run'ов — один общий write всего участка (плюс тест `nearby_delta_runs_are_written_as_one`).
- Замер нового кода: max copy-back упал **1751 → 134 мс** (провал relay устранён), но world median 225,3 мс против 161,4 у `DELTA_LAND=0` → **регрессия по кадрам осталась**, поэтому `DELTA_LAND` оставлен **выключенным по умолчанию** (включать только после доработки).
- `BAKE_TEXTURES_MS=25` закреплён по умолчанию в `run-road.ps1` (в `launch-game.ps1` такой ручки нет — там дефолт кода).
- Масштабирование бюджетов 8 ГБ (101Leafy #26: RT-резерв 3072 MB, spares 4 ГиБ) **не перенесено** — требует правок `resource.rs`/`pool.rs`, следующий сеанс.

**B2 (lit-фолбэк shadow-mask) — не реализован.** Для него нужен `diag`-билд с `VICTORY_CAPTURE` (feature `diagnostics`; отдельный ключ кэша шейдеров → прогрев) и дамп классов `R32_UINT`-таблиц; заготовка логирования уже в `capture.rs` (Шаг A). Это первый шаг следующего сеанса.

**B4 (закреплено):** свет по умолчанию `AMBIENT=0.5`, `AMBIENT_SPECULAR=0.3,0.15` (в обоих скриптах), `NATIVE_1440P=1` + `-RenderScale 0.5` (нормальный 2K-UI: «ВЫБОР УЧЁТНОЙ ЗАПИСИ» 246×30 px против 355×38 при 1080p), `FPS_COUNTER=all`, `TILES_SMALL=512`, `VICTORY_DLSS=0`. Региональные метрики `r12-analyze.ps1` по кадрам B4 ещё не снимались (следующий сеанс вместе с B2 и `RT_SHADOW=lit`).

**Факты для следующего шага:** провал `r12-run2` — это не диск/pagefile (active time 1 %, avg 0.9 мс, pagefile 0.99/5.5 ГБ): узкое место — GPU 0,9–1,3 с + relay copy-back при полной VRAM (5,86/8,19 ГБ, host-копии 1018 MB). Приоритеты: бюджеты 8 ГБ (#26) и перекрытие copy-back с GPU, затем B2.

## 36. Раунд 13 — бюджеты 8 ГБ (`101Leafy #26`), честный 1080p-рендер (`RENDER_SCALE=0.75`), `VICTORY_RT_SHADOW=lit`, полная телеметрия

Ветка **`main-3060ti`**. Коммиты: **`c42ffbb`** (бюджеты), **`3b4208c`** (`VICTORY_RT_SHADOW=lit`), этот docs. `victory-shader` не тронут.

### 36.1. Радар (read-only, 5+ репозиториев)
- **101Leafy/victory-leafy PR #26** (`d4aff42e4e`, open, CONFLICTING): ресурсная половина перенесена; `pool.rs`-половина (spares, до 64×64 MiB) **не переносима** — у нас нет `pool.rs` (есть только `shadow::card_room_kept` + 256 MiB staging spares). Оговорка автора PR: на 7 122 MB бюджета и 2 499 MB живых таргетов резерв всё равно упирается в 0 — на нашем пике 5.8–6.7 GB выигрыш может не связать; в прогонах «no room on the card» остался (44 строки/прогон).
- **trustno1 #19** (`0a8cd3ef5e`, docs): «тёмные комнаты — это пропорциональный bound оценки ambient-стенд-ина, а не свет-ручка» → отсюда реализация `RT_SHADOW=lit` (ниже); таблица shadow-mask при выключенных RT-трассерах не пишется вовсе.
- Прочее: Belobug #540 (ambient-фолбэк не готовит BVH-пайплайны), #223 (room_percent vs vram_keep), fxceless #1466/#1431 (copy-back/landing), victory-leafy #24 (CSV-гейджи — телеметрия под #26), Marvisak #3457 (LRU shadow-бюджет). trustno1: новых DLSS/RT-коммитов после `3f60eeee0b` нет.
- Важно: у Marvisak нет мерджей с 2026-11-26, ~50 открытых PR — необзоренный поток; trustno1 `826ba150c8` уже портирован (Раунд 12).

### 36.2. Шаг 1 — бюджеты под 8 ГБ (`c42ffbb`)
`resource.rs`: `targets_room_most(budget, asked)` — резерв `min(3072 MB, budget/4)` → на 8 GB-карте **2 GiB** (реальный DXGI-бюджет 7.1–7.4 GiB → ~1.8 GiB) вместо плоских 3072 MB; env `VICTORY_CARD_ROOM_FOR_TARGETS_MB` остаётся override. Прокинут в `room()`, `host::readonly_upload`/`rebar_copy`; тест `the_targets_reserve_follows_the_card_budget` (24/12/8/4 GiB). Гейт: check 0, test 0 failed (флейк §23.3 пропущен).
Телеметрия: `audit/round13/r13-analyze.ps1` — процентили frametime/FPS min/p50/p90/p95/p99/max, фризы >150/>500 мс, стадии preparing/copies/textures/GPU/copy-back, dropped/indirect, «no room on the card», tiles_black, readback reused/made, VRAM peak/mean, GPU util/clock/power, топ CPU-потоков, commit.

### 36.3. Шаг 2 — `VICTORY_RT_SHADOW=lit` (`3b4208c`)
Реализован как **мягкий lit-уровень в нашем ambient-стенд-ине** (`ambient.rs::raise`), а не правка таблицы масок: по trustno1 #19 тьма идёт от пропорционального bound оценки (`found >= 0.2*all` → 0 в тёмной комнате), а сама таблица при выключенных трассерах не пишется. При `VICTORY_RT_SHADOW=lit` (или числе 0.01–2, default share 0.35) к `was` добавляется `found * share` — комнаты/лица становятся тусклыми, не угольными. **Off по умолчанию**, тест парсинга. Гейт: check 0, test 0 failed.
Диагностический дамп классов `R32_UINT`-масок **не снимался** (нужен diag-билд + прогрев его кэша) — следующий сеанс.

### 36.4. Шаг 3 — релиз и два тихих прогона (240 с, `NATIVE_1440P=1`, `RENDER_SCALE=0.75` = 1920×1080 честного рендера)
Релиз 9 341 952 B. Скрипты: `RenderScale=0.75`, `AMBIENT=0.5`, `SPECULAR=0.3,0.15`, `BAKE=25`, `FPS_COUNTER=all`, `WATCH_STATS=1`, `TILES_SMALL=512`, тихий харнесс.

| метрика (world) | Run 1 `RT_SHADOW=off` | Run 2 `RT_SHADOW=lit` |
|---|---|---|
| кадры n | 162 | 171 |
| p50 | **7 fps / 144,4 мс** | **8 fps / 132,4 мс** |
| p90 | 294,3 мс | **540,4 мс** |
| p95 / p99 | 350,6 / 474,8 | 555,4 / 855,1 |
| max | 2 245,0 мс | **7 893,0 мс** |
| фризы >150 / >500 мс | 70 / **2** | 75 / **22** |
| стадии p50: preparing / copies / textures / GPU / copy-back | 97,2 / 64,6 / 8,6 / 15,7 / 47,8 мс | 74,7 / 48,4 / 7,6 / 13,2 / 31,5 мс |
| GPU p90 | 28,9 мс | 74,1 мс |
| dropped p50/p90 | 114 / 521 | 107 / 404 |
| VRAM peak / mean | 7 524 / 5 744 MB | 7 544 / 5 897 MB |
| GPU util p50/p90 | 38 / 69 % | 34 / 65 % |
| процесс commit | 20 026 MB | 19 120 MB |

Прогоны **не сцен-матчены** (разные сцены/маршруты), поэтому A/B `lit` vs `off` по кадрам неубедителен: в run2 тяжелее сцена (max 7.9 с, p90 540 мс), при этом copy-back/подготовка в среднем дешевле. **`RT_SHADOW` остаётся opt-in (off by default)** до сцен-матча и региональных метрик; бюджеты 8 ГБ оставлены включёнными (регрессии по кадрам не видно, «no room» сохраняется, но это отдельная причина — тайлы/сцена).

### 36.5. Осталось
1. Сцен-матч A/B `RT_SHADOW=off/lit` + региональные метрики (скрипт `r12-analyze.ps1` при вызове из нового анализатора отдал только баннер — починить вызов/пути).
2. diag-билд + прогрев кэша + `VICTORY_CAPTURE` → дамп классов масок и non-finite экспозиции; по нему решить, нужен ли отдельный mask-фолбэк поверх lit-флора.
3. Бюджеты: проверить, снизились ли перезаливы тайлов на сцене у машины копов при равных `draws`-корзинах; кандидаты: victory-leafy #24 (CSV-гейджи), Belobug #540, #223, fxceless #1466/#1431.
4. `VICTORY_DELTA_LAND` (батчевый) — довести до отсутствия регрессии, иначе оставить off.

## 37. Раунд 14 — батчевый DELTA_LAND (3 KiB/8 вызовов), порты радара (#1431, #3429), сцен-матчевый A/B по корзинам `draws`

Ветка **`main-3060ti`**. Коммиты: **`281e706`** (батчевый landing: склейка до 3072 Б, ≤8 `write_mapped` на piece), **`b6074b5`** (#1431 `joined()` in place + #3429 счётчики VirtualQuery — без изменения поведения), этот docs. `victory-shader` не тронут.

### 37.1. Радар (read-only, 6+ репозиториев)
- **fxceless #1431** (`9342db15f0`, open): `shadow::joined` — тот же sorted union без второй аллокации (upstream 2.2→1.4 µs на 512 диапазонов) — **портирован**.
- **Marvisak #3429** (open): копировальный терм `VirtualQuery` был невидим в статистике (`C::Queries/QueryUs` никогда не инкрементировались) — **портирован** (telemetry only).
- **fxceless #1466** (`0e2d0a9bb0`): SKIP — в нашем дереве нет механизма `PULL_AFTER/take_written/Shadow::pull_back` (наш copy-back — readback-тикеты `backreads.rs/texback.rs`), это не «та же страница ещё раз».
- **Belobug #540** (`af5bdc9720`): ambient-фолбэк не должен готовить BVH-пайплайны; у нас `ambient::on()` зовёт `builder_runs()`→`make::now` (синхронная компиляция), но `VICTORY_BVH_BUILDER` по умолчанию off → латентно, отложено (STUDY, кандидат на следующий раунд вместе с RT-трейсом).
- **Belobug #223** (SKIP: у нас нет `vram_keep/keep_shown`; наш 8 GiB policy уже 94% + четверть-бюджетный резерв), **Marvisak #3457** (SKIP: нет `bindless.rs`/`evict_unused`), **#3437** (STUDY: у нас уже есть `VICTORY_WINDOW_KB`=4 MiB), **101Leafy #24/#30** (STUDY: CSV-гейджи, частично портируемо). trustno1: после `3f60eeee0b` новых copy-back/landing/`write_mapped`/pacing работ нет; HEAD `959dcaf609` — только mouse/HAGS и #29 (violet probes, уже портирован в Раунде 12).
- Важно: у fxceless #1466/#1431 статус OPEN (не влиты); числа upstream — их, не наши.

### 37.2. Шаг 1 — код
- `shadow.rs`: `DELTA_GAP 512→3072`, `DELTA_RUNS 64→8` — меньше системных вызовов на штормах; тест обновлён (2048..=4096, ≤8). Гейт: check 0/test 0 failed (флейк §23.3 пропущен).
- `shadow.rs::joined()`: `sort_unstable`+`dedup_by` на месте (порт #1431, GPL-3.0-or-later), без второй аллокации; `host.rs::read_sparse`: `C::Queries/QueryUs` вокруг `VirtualQuery` (порт #3429) — только диагностика.

### 37.3. Шаг 2 — релиз и A/B (`NATIVE_1440P=1`, `RENDER_SCALE=0.75`, 240 с, телеметрия `all`+`WATCH_STATS=1`)
Релиз 9 342 848 B. Прогоны: `road\r14-run1` (control: `RT_SHADOW=off`, `DELTA_LAND=0`) и `road\r14-run2` (`RT_SHADOW=lit`, новый `DELTA_LAND=1`).

| метрика (world) | Run 1 контроль | Run 2 lit+батч |
|---|---|---|
| кадры n | 130 | 177 |
| p50 | **8,4 fps / 119,5 мс** | **5,0 fps / 200,0 мс** |
| p90 / p95 / p99 | 934,3 / 1 440,5 / 1 876,3 мс | 539,2 / 626,5 / 940,4 мс |
| max | **7 501,0 мс** | 2 560,9 мс |
| фризы >150 / >500 мс | 61 / 28 | 114 / 26 |
| VRAM peak / mean | 7 547 / 5 999 MB | 7 546 / 5 982 MB |
| GPU util p50/p90 | 36 / 50 % | 34 / 53 % |
| процесс commit | 19 110 MB | 19 824 MB |

Топ-5 фризов:
- run1: **7501 мс** (GPU=473,6; making 94,0; copies 61,5 ms/119 MB; textures 9,4 ms/15,8 MB; back 20,3), 1440,5 (GPU 50,9), 1365,6 (**GPU 925,6**), 1320,2, 934,3;
- run2: 2560,9 (making 178,7; copies 115,0/112,9 MB; textures 96,3/180,9 MB; GPU 109,7; back 68,9), 1343,8 (copies 157,1/197,8 MB), 940,4 (textures 126,3/212,5 MB), 595,3, 541,2 (back 139,6, dropped 922).
- CPU: у run1 самый занятый поток — victory-heap 46%, у run2 — victory-submit 77% (в разные моменты).

**По равным корзинам `draws`** (A=run1, B=run2): 500: +56 мс; 600: +39; 800: **-56**; 900: +41; 1000: +27; 1100: **-456**; 1200: **-184**; 3500: **-81** — знаки смешанные, сцены не совпадают (n по 3–43), поэтому **ни `RT_SHADOW=lit`, ни `DELTA_LAND=1` не закреплены по умолчанию** (оба остаются opt-in), регрессии по коду нет, но и подтверждённого выигрыша нет.
Региональные метрики яркости (face/water/beach/sky, mean) на одинаковых кадрах: shot-45 у обоих совпадает (0,629/0,523/0,588/0,448); дальше сцены разные (run2 shot-105 темнее: face 0,140 против 0,537) — сравнение не показательно.

### 37.4. Закрепление и осталось
- Профиль рендера/телеметрии не менялся: `NATIVE_1440P=1`, `RENDER_SCALE=0.75`, `FPS_COUNTER=all`, `WATCH_STATS=1`, `BAKE=25`, `TILES_SMALL=512`, тихий харнесс; дефолты `RT_SHADOW`/`DELTA_LAND` — off.
- Телеметрия: `audit/round13/r13-analyze.ps1` теперь сам считает процентили, фризы >150/>500 мс, **топ-5 самых долгих кадров с полной разбивкой** (making/copies MB/textures MB/GPU/back/dropped) и **региональные метрики яркости** по скриншотам (face/water/beach/sky, mean/p50/dark%), плюс «no room», tiles_black, readback, VRAM/GPU/CPU, commit.
- Осталось: сцен-матч с фиксированной камерой (сейчас сцены разные), diag-дамп масок, порт Belobug #540 (латентный), разбор victory-submit 77% и 7,5-секундного фриза run1 (GPU 473 мс + неучтённое ожидание), бюджеты #26 проверить на равных корзинах.

## 38. Раунд 15 — фикс-камера, порт fxceless #1442 (named pipeline loads), диагностика медленных кадров, ручной сцен-матч-прогон (окно 388–900 с)

Ветка **`main-3060ti`**. Коммиты: **`940dfdc`** (`builder_whole` — порт Belobug #540 — и лог медленных кадров), **`a1e7c97`** (порт fxceless #1442: именные загрузки пайплайнов), этот docs. `victory-shader` не тронут.

### 38.1. Радар (read-only)
- **fxceless #1442** (`+64/−9`, `pipeline/library.rs`): 64-страйповая блокировка имён → глобальный `HashSet<String>`+`Condvar`: разные имена больше не ждут друг друга (upstream: 7.6 с и 14.8 с ожидания submit-потока) — **портировано** (`a1e7c97`, два теста).
- **fxceless #1450**: предел ожидания compute-PSO (457 compute PSO × ~1 с в одном кадре → watchdog) — следующий кандидат, у нас `make::ready()` без cap.
- **Belobug #540**: у нас уже в HEAD (`940dfdc`, `builder_whole()` вместо `builder_runs()` в `ambient::on()`; `VICTORY_BVH_BUILDER` по умолчанию off → латентно).
- Остальное: #3516 (guard вне lock — у нас иной layout), #3485 (`VICTORY_WATCH_REGION_PROBE`, opt-in A/B), #3487 (batched submit heap — конфликтует с нашим memory.rs), 101Leafy #28 (лог «frame stopped waiting for pipelines» — дешёвый), #24 (CSV-гейджи), trustno1 #34 (placed constants/order, меняет default), fxceless #1302/#1454 (детерминированный harness/`VICTORY_KEYS_ONLY`), Belobug replay-подсистема.
- Названные в брифе `VICTORY_ROOT_BIND_CACHE` и `VICTORY_TEXTURE_STAGE_PER_FRAME` **уже есть** в дереве (`draw.rs:595`, `draw.rs:2519`, внесены в Раунде 8, off по умолчанию) — A/B по ним не делался.

### 38.2. Фикс-камера и харнесс
- `run-road.ps1`: `-FixedCamera 1` (по умолчанию) — статический `VICTORY_KEYS`-маршрут. **Первая версия была неверной**: маршрут не скипывал пролог (>3 мин катсцен), и прогоны 240 с заканчивались внутри катсцен — эти 3 прогона (`r15-run1..3`) для A/B не засчитаны.
- Исправление: Enter-пульсы до 180 с (скип пролога), затем только микро-нудж `L` каждые 45 с; `-FixedCamera`-прогоны — `-Seconds 420+`, фиксированное окно — последние ~4 минуты.
- Финальный замер сделан **вручную**: пользователь сам скипывал катсцены, `VICTORY_KEYS` не инжектировался (`-Keys '999:Enter'`, `-FixedCamera 0`), прогон `road\r15-manual` 900 с, `RT_SHADOW=lit`, старт геймплея отмечен пользователем (18:33:44 = **388-я секунда**).

### 38.3. Диагностика медленных кадров
`host.rs`: для кадра >300 мс пишется `slow frame: the stages above do not account for all of it ms=… accounted=… unaccounted=… fps=… draws=… making=…`. В катсценовых кадрах ручного прогона `unaccounted` = 32–1020 мс (т.е. известные стадии объясняют меньшую часть), `making=0` (PSO не в этом кадре); в игровом окне после стриминга медленных кадров с >300 мс почти нет (max 414 мс).

### 38.4. Итоговый ручной прогон (anchor 388 с)
| метрика | весь прогон (world) | окно **388–900 с** |
|---|---|---|
| кадров | 725 | **536** |
| p50 | 118,2 мс (8,5 fps) | **116,3 мс (8,60 fps)** |
| p90 | 245,3 мс | **145,7 мс** |
| p95 | 411,5 мс | **154,2 мс** |
| p99 | 1 065,7 мс | **224,1 мс** |
| max | 2 286,6 мс | **413,8 мс** |
| фризы >150 / >500 мс | 37 / 15 | **42 / 0** |
| draws median | — | **3 878** (лодочная миссия) |
| VRAM peak / mean | 7 486 / 6 671 MB | — |
| readback reused/made | 78 207 / 53 | — |
| «no room on the card» | 166 строк | — |

После прогрева картинка и кадры стабильны: **8,6 fps медиана, p99 224 мс, ни одного фриза >500 мс**; узкое место — CPU (submit/heap 46–70 % одного потока, GPU 50–78 %), а не GPU и не диск.

### 38.5. Артефакты и вода (скрины — `audit\round15\artifacts\`)
- **Белая вода на прогреве**: `shot-330` и `shot-360` (a1/a2) — вода выжжена в белый, пока не догрузились текстуры/свет; к 390-й секунде проходит (a3/a4).
- **Игровое окно 388–900**: вода тёмно-синяя (navy), волны/блики/отражения корректные, без чёрных тайлов (a5/a6: shot-465, shot-885).
- **Эталон светло-голубой воды** (как ожидает пользователь) найден только в Series S-прогонах Раунда 6: `road\ss-patched2\shot-419/434/449_f1` (r0–r2 в artifacts) — бирюзовая вода, но **с чёрными тайлами**. На Series X вода стабильно темнее.
- Гипотезы разницы: (1) на Series X выключена water tiled-light программа `5dc3b259…` (иначе тёмные тайлы); (2) нет RT-отражений/ambient → вода темнее; (3) экспозиция/тонемап Series S-пути. Следующий A/B: `VICTORY_WATER_LIGHT=1` и `VICTORY_AMBIENT_SPECULAR_ALPHA=1` на этой же лодочной сцене (90–120 с после прогрева).

### 38.6. Закрепление и осталось
- Профиль не менялся: `NATIVE_1440P=1`, `RENDER_SCALE=0.75`, `AMBIENT=0.5`, `SPECULAR=0.3,0.15`, `BAKE=25`, `FPS_COUNTER=all`, `WATCH_STATS=1`, `TILES_SMALL=512`; `RT_SHADOW` и `DELTA_LAND` — opt-in; `-FixedCamera 1` — дефолт харнесса (с исправленным скипом пролога).
- Осталось: порт #1450 (cap ожидания compute-PSO), A/B воды (`WATER_LIGHT`/`SPECULAR_ALPHA`), fidelity-прогон без скипа катсцен (600 с), валидация #1442 на равных корзинах, бюджеты #26 на равных корзинах.

### 38.7. Цвет воды: сравнение с PS5 Pro и Series S (пост-раунд 15)

Пользователь дал скрин геймплея того же лодочного задания с **PS5 Pro** (сохранён как `audit\round15\artifacts\ref_ps5pro_water_scene.png`). Цвет воды измерен в двух зонах без лодки/миникарты (средние RGB):

| кадр | левая вода | правая вода (блики) |
|---|---|---|
| **PS5 Pro (референс)** | **#658A89** (101,138,137) | #B9CCBA |
| Наш Series X (`r15-manual` shot-465) | **#3F5368** (63,83,104) | #969A99 |
| Наш Series S (`ss-patched2` shot-419) | **#609081** (96,144,129) | #62947B |
| Наш белый артефакт (`r15-manual` shot-360) | #959B9E | #EEECE9 |

Вывод: по оттенку **Series S-прогоны почти совпадают с PS5 Pro**, а наш текущий Series X **синее и темнее примерно на 40 %** по яркости. Наиболее вероятная причина: Series S-прогоны — Раунд 6, **до** `VICTORY_AMBIENT_SPECULAR` (появился в Раунде 10); тогда вода получала родные небесные отражения заголовка (`CS_RayMiss` → кубмапы неба). Сейчас `AMBIENT_SPECULAR=0.3,0.15` заменяет ~85 % неба на нашу (тёмную на воде) оценку — вода уходит в тёмно-синий.

**Следующий эксперимент (Раунд 16, ручной сеанс — лодочная миссия требует игры):** три плеча по 90–120 с после прогрева, катсцены пользователь скипывает сам, затем одинаковое положение на лодке; мерить те же зоны воды и сверять с `#658A89`:
- **A (гипотеза):** `VICTORY_AMBIENT_SPECULAR=0` — родное небо заголовка, ожидаем бирюзу; риск — «небо везде» в отражениях без BVH;
- **B:** `VICTORY_AMBIENT_SPECULAR=0.3,0.85` — 85 % неба, 15 % оценки; ожидаем светлый оттенок без грязных отражений;
- **C:** текущий `0.3,0.15` — контроль.

Если A/B дают бирюзу — менять дефолт `AMBIENT_SPECULAR` на «небо почти не трогать» (напр. `0.2,0.9`) и проверить другие поверхности (стекло/машины) на артефакты. Эталонные/артефактные кадры: `audit\round15\artifacts\a1..a6`, `r0..r2`, `ref_ps5pro_water_scene.png`.

---

## 39. Раунд 16 — глубокий разбор `trustno1-coder/VIctory-NVIDIA` / `foxhound` и изолированный прогон на RTX 3060 Ti 8 ГБ (27.11.2026)

Ветка `main-3060ti`, база `f503194`. Чужой репозиторий склонирован в `C:\GTA6_Port\victory-nvidia-test` (`main` = `d12eb1b`; база foxhound `02fc18a`, merge `4db4277`, 22 first-parent коммита поверх). **Основной репозиторий и кэш не тронуты**: `victory-fork` чист на `f503194`, `shaders-r8-group2` — 27 180 файлов, последняя запись 27.11 14:29 (до начала тестов); `victory-shader` не менялся.

### 39.1. Шаг 0 — два read-only аудита (готовы, в репо не правилось)

`audit\round16\foxhound-analysis.md` (733 стр., 127 КБ) — архитектура ускорений foxhound; `audit\round16\victory-nvidia-analysis.md` (475 стр., 70 КБ) — фичи форка, DLSS, вода, 8 ГБ, портируемость. Оба — по коду и докам с точными `файл:строка`; сборок/запусков в них не было.

Главное:
- **«Memory compares moved to the card»** (11→22 FPS) — GPU-хеширование 64-КиБ кусков карточных копий титула (`CARD_COMPARE`, их `host.rs:2137`), фетч только изменённых, merge. **В нашем дереве механизм уже есть** (та же fxceless-линия: `host.rs:1849/1903/2003/2070`, причём с fetch внутри полёта). Для нас это «уже в базе», а не источник прироста.
- **«dropped draws с тысяч до ~0»** — 1 000 000-лимит shader-visible view-хипа на NVIDIA; их фикс `DYNAMIC=2^18` + вырезание «кольца» title-хипа. **У нас сделано дальше и измерено на 8-ГБ карте**: 606 784 динамических слота, 0 отказов (`descriptor\host.rs:1003-1053`). Переносить нечего; полезен только багфикс `47747d6` (слоты возвращаются только своему хост-хипу).
- Реально новые/переносимые механизмы (всё в `victory-gpu`, шейдеры целы): приоритет helper-потоков submit'а (`80c70b7`; их замер на i5-12400F 6C/12T — 8.6→9.6 FPS), async/параллельный `sync_heaps` (`HEAP_SYNC_ASIDE/EARLY`, `SYNC_IN_PLACE`; у нас heap-pass 14.9 мс на submit-потоке), многопоточный лендинг `land.rs`+`safecopy` (сравнение 32 байта, запись только изменённых dword; у нас лендинг ~99.7 мс/кадр), `VICTORY_CARD_STEPS`, `VICTORY_WC_NO_BACK`, `VICTORY_SOFT_VIEWS`, `VICTORY_LEAVE_OUT` (heat-haze + дефрагментация decal-heap), `aec85ef` (raw-view clamp), BC7-rotate, `SKIP_BINK`, и крупное — **DLSS-движок Streamline** (`6395727`, ~2–3 тыс. строк адаптации).
- Не переносится: `VICTORY_THIN` (977+ строк, NO-GO; на 3060 Ti чужой прогон давал 2–3 fps), `PORT_CONSTANTS` (на малой VRAM GPU ждёт), thread-ticket `854f57f` (меняет `victory-shader` → новая ревизия кэша; только отдельной веткой), FrameGen (RTX 40+), их 26–30 FPS/12 ГБ как ожидание.
- План аудита — **A-lite**: точечные порты к нам тремя волнами (39.7). Вариант «перенести наши фичи к ним» отклонён: повторный порт `victory-shader` (~8.8 тыс. строк), потеря прогретого кэша, сброс baseline, 8 ГБ у них не проверялись.

### 39.2. Шаг 1 — изолированная сборка и харнесс

- **Изоляция** (проверена по факту): папка `C:\GTA6_Port\victory-nvidia-test`; на каждый запуск `LOCALAPPDATA=C:\GTA6_Port\victory-nvidia-appdata`, `VICTORY_CACHE_DIR`/`VICTORY_DATA_DIR` туда же. В `data` скопированы только сейвы и настройки профиля (2.3 МБ, EULA принят) — без кэша.
- **Сборка**: `cargo build --release --locked -p victory` — **1 м 46 с** (rust 1.98.1), бинарь 10 795 008 Б. `victory.toml` → `game_dir='C:\GTA6_Files'`.
- **Шейдеры**: `victory.exe shaders` (с `VICTORY_HOST_WAVE=32`) — **25 760 переведено за 333 с**; 225 не переводятся (198 «контейнер без программы», 22 «no wave32 lowering for this wave64 program», 5 «descriptor read only as data»).
- **`warm`**: бук записал **83 568 пайплайнов**, `warm` шёл ~10 пайплайнов/с → ~2.2 ч; остановлен. Прогретый драйверный кэш PSO от холодного прогона при этом остался, поэтому второй прогон — «тёплый».
- **Харнесс**: `audit\round16\run-nvidia.ps1` — та же механика, что `run-road.ps1` (тихо: без `HWND_TOPMOST`, без физ. ввода, окно 1280×720 `HWND_NOTOPMOST`+`SWP_NOACTIVATE`, вход только `VICTORY_KEYS`). Профиль — `scripts\play-foxhound.ps1` с `-Dlss off|quality -Res 1080to1440` и **без frame generation** (`-NoFrameGenReady`: на 8 ГБ не отдаём 14 % бюджета под недоступный FG). DLSS проверен заранее: `upscaler-check --device` — SR работает (SL 2.14.16, ngx 310.9.16), FG «not supported on this GPU».
- **Сцена**: `VICTORY_KEYS`-маршрут (Enter-пульсы) проходит пролог и приводит на **ту же лодочную миссию** («Следуйте за самолётом»), что и ручной прогон §38.4; окно геймплея — 340–420 с. Образ опознан их патчем: `victory-image-b99f876b74d4a77f.dll` (тот же дамп v1.0.2.1).

### 39.3. Прогоны (по 420 с; окно геймплея: A1 389–420 с, A2/B 340–420 с)

| | A1 холодный | A2 тёплый | B DLSS quality |
|---|---|---|---|
| DLL/профиль | DLSS off, 1080→1440 | DLSS off, 1080→1440 | DLSS quality (рендер 1706×960) |
| fps p50 (окно) | **12,24** | **11,95** | **13,28** |
| frame ms p50 / p90 / p99 / max | 81,7 / 87,0 / 90,9 / 91,8 | 83,7 / 97,9 / 113,8 / 136,4 | 75,3 / 83,4 / 87,9 / 90,7 |
| freezes >150 / >500 мс (окно) | 0 / 0 | 0 / 0 | 0 / 0 |
| draws p50 (окно) | 4 296 | 3 881 | 3 904 |
| VRAM peak / mean | 7 700 / 6 644 MB | 7 944 / 6 846 MB | 7 551 / 6 476 MB |
| GPU util p50 | 95 % | 86 % | 93 % |
| dropped draws p50 (окно / весь прогон) | — / 550 | 85 / 89 | 78 / 83 |
| весь прогон: ms p50 (world) | 288,8 (293,3) | 101,1 (281,9) | 84,1 (88,2) |

- Ни одного «device lost», ни одного «no room on the card»; ошибок нет. A1→A2 прогрев снизил dropped draws p50 550→89 и убрал прологовые стопы, но **геймплейный fps не изменил** (12,24→11,95): в геймплее пайплайны уже скомпилированы и в холодном прогоне.
- **Катсцены/пролог медленные у обоих** (3–4 fps, 280–330 мс, GPU/VRAM 7.6–7.9 ГБ): их путь там не спасает; в нашем §38 подобные кадры были 300–1000 мс. Это отдельная проблема стриминга/VRAM, не различие форков.
- DLSS: **`1.00 upscalings by DLSS, 0.00 failed, 0.00 left to the title's FidelityFX`** — 2 961 upscale, ни одного отката на родной FSR, картинка чистая; FrameGen корректно недоступен и не включался. Побочно: рендер 1706×960 вместо 1920×1080 даёт **−400 МБ VRAM** (7551 против 7944) и **+11 % fps** к их же сборке без DLSS.
- Их баги, видимые в логе: 1 compute-шейдер `6b82bc7d22016cb5` не переводится («no wave32 lowering») → его draw'ы дропаются (это и есть dropped ~80/кадр); 107 предупреждений «texture picked by index changed after its upload»; view-heap обрезан на 1 000 000 при запрошенных 1 245 184 view (245 тыс. clamp'нуты в null-слот).

### 39.4. Лобовое сравнение с нашей `main-3060ti` (равные корзины draws, та же лодочная миссия)

Наш ручной прогон §38.4 (окно 388–900 с) против их A2/B (окно 340–420 с). `draws` совпал почти точно (3 877 против 3 881/3 904), сцена та же.

| корзина draws | наша сборка, мс / fps | их A2, мс / fps | их B (DLSS), мс / fps |
|---|---|---|---|
| 3700 | 116,0 / 8,62 | 87,5 / 11,43 | 72,2 / 13,85 |
| 3800 | 112,1 / 8,92 | 84,6 / 11,82 | 73,5 / 13,61 |
| **3900** | **116,3 / 8,60** | **81,1 / 12,33** | **76,4 / 13,09** |
| 4000 | 117,7 / 8,50 | 86,8 / 11,52 | 75,8 / 13,19 |
| 4100 | 117,7 / 8,50 | 84,2 / 11,88 | 75,5 / 13,25 |
| 4200 | 117,1 / 8,54 | — | 76,3 / 13,11 |
| **p50 всего окна** | **116,3 / 8,60** | **83,7 / 11,95** | **75,3 / 13,28** |
| p99 | 221,0 | 113,8 | 87,9 |
| фризы >150 мс | 42 из 536 | 0 из 81 | 0 из 79 |

**Стадии в тех же окнах (p50 из их же строчки кадра):**

| стадия, мс | наша | A2 | B |
|---|---|---|---|
| кадр | 114,5 | 84,0 | 72,6 |
| preparing | 85,3 | 54,3 | 44,1 |
| copies | 36,1 | 18,7 | 14,5 |
| textures | 10,5 | 1,5 | 1,5 |
| «на GPU» | 14,5 | 3,4 | 2,9 |
| copy-back | 55,7 | 21,4 | 15,3 |
| dropped | 80 | 85 | 78 |

Честные оговорки к таблице: (1) у нас включён `VICTORY_FPS_COUNTER=all` (внутриигровой бокс, по нашим же замерам Раунда 11 ~21 мс/кадр) и `RT_SHADOW=lit`, у них ни того, ни другого; без бокса наша оценка ~93–95 мс → реальный разрыв **≈1,2–1,5×**, а не 1,55×; (2) они **выкидывают пассы** (`VICTORY_LEAVE_OUT`: heat-haze `731a39171b568298` + дефраг decal-heap `02384a60fda6f2d0`, −5.6 мс GPU по их замеру) и дропают ~80 draw/кадр, т.е. их картинка дешевле; (3) у нас GPU 61–63 % (упор в CPU-поток submit), у них GPU 86–95 % — **их CPU-путь быстрее, и это совпадает с их же findings** («the thread that prepares submissions: 48 ms. It is the wall»).

### 39.5. Вода: их форк её НЕ исправляет (важно)

Те же зоны кадра (левая вода без лодки/миникарты, правая с бликами), средние RGB:

| кадр | левая вода | правая вода (блики) | небо |
|---|---|---|---|
| PS5 Pro (референс) | **#628985** (98,137,133) | #BDCDBA | #BBC9D4 |
| наша сборка (`r15-manual` 465/885) | #3C556A (60,85,106) / #384E63 | #959490 / #8E8E89 | #CFD9E3 |
| их A2 (390/405) | #3A5267 (58,82,103) / #3D5468 | #9C9B95 / #989893 | #D1DAE4 |
| их B DLSS (392/407) | #3B5368 (59,83,104) / #384F64 | #8A8C88 / #90908B | #D3DCE5 |

Вывод: **вода у них такая же тёмно-синяя, как у нас** (разница ≤4/255), хотя у них `AMBIENT=0.8 / AIR=1.5 / SPECULAR=1` против наших `0.5 / 1.2 / 0.3,0.15`, и небо совпадает почти точно. Значит гипотеза §38.7 «`AMBIENT_SPECULAR` затемняет воду» **не подтвердилась**: перевод неба из 15 % в 100 % не двигает оттенок. Причина у обоих одна и та же — **два tiled-шейдера света воды (`5dc3b25986622949`, `6f225b641c5b0708`) выброшены** (`left_out`), а без них, по их же README/меню, вода «darker and bluer», и `VICTORY_WATER_LIGHT=1` подписан как «ON (GREEN, DARK BLOCKS)». Т.е. это не различие форков, а общий незакрытый пункт: **плечо D Раунда 17 — `VICTORY_WATER_LIGHT=1`** (ожидаем зелёную составляющую + тёмные плиты + ~15 мс GPU); плечи A/B/C по `SPECULAR` остаются, но с низким приоритетом.

### 39.6. Вердикт и план Раунда 17

1. Их сборка на нашей карте **быстрее в геймплее**: 11,95 fps без DLSS и 13,28 fps с DLSS против наших 8,60 fps в равных корзинах `draws`; у нас упор в CPU (GPU 61 %), у них — уже в GPU. Но два «ускорителя», которыми автор объясняет 2×, у нас **уже есть**; разница — в их архитектуре лендинга/копий/очередей и в выкинутых пассах.
2. Порядок портов в `main-3060ti` (каждый — отдельный коммит, `cargo check/test`, A/B на равных корзинах, `victory-shader` не трогать):
   - **P0** `VICTORY_LEAVE_OUT`-минимум: heat-haze `731a39171b568298` + дефраг decal-heap `02384a60fda6f2d0` через наш `pipeline::left_out` (~10 строк; GPU −5.6 мс/кадр по их замеру);
   - **P0** `VICTORY_WC_NO_BACK` (наш copy-back 55,7 мс/кадр против их 21,4);
   - **P1** async/параллельный heap sync (наш heap-pass 14,9 мс на submit-потоке);
   - **P1** лендинг в духе `land.rs` (наш 99,7 мс → их 58–60);
   - **P2** `47747d6` (id хост-хипа), raw-view clamp `aec85ef`;
   - **волна 3** DLSS-движок: главный риск снят — у них **100 % перехват upscale'ов и 0 откатов на наш падающий FSR4-путь**; гейт приёмки — тот же лог «0 left to the title's FidelityFX»;
   - helper-priority — только отдельным A/B (аудиты расходятся: один оценивает +8–12 %, другой — конфликт с нашей thread-моделью).
3. Воду порты не лечат: обе сборки тёмно-синие. Раунд 17 — плечо D (`WATER_LIGHT=1`) до всех крупных портов, т.к. это дешёвая проверка цвета.

### 39.7. Артефакты и что осталось

- Аудиты: `audit\round16\foxhound-analysis.md`, `victory-nvidia-analysis.md`.
- Скрипты: `run-nvidia.ps1`, `r16-compare.ps1` (равные корзины + окна), `r16-water.ps1` (RGB зон), `r16-frames.ps1` (стадии по окну).
- Логи: `build-release.log`, `shaders.log`, `warm.log`, `analyze-r16-a1/a2/b.log`, `compare-r15-vs-a2.log`, `compare-r15-vs-b-dlss.log`, `compare-a2-vs-b-dlss.log`.
- Прогоны: `road\r16-a1` (холодный), `road\r16-a2` (тёплый), `road\r16-b` (DLSS); скрины — `audit\round16\artifacts\` (`a1_cold_gameplay_390.png`, `a2_warm_gameplay_390/405.png`, `a2_cutscene_slow_240.png`, `b_dlss_gameplay_392/407.png`).
- Не сделано: `warm` (83 568 пайплайнов, ~2.2 ч — нецелесообразно); их `-Res 1440` (native-рендер без 1080p) не прогонялся; сравнение с нашей сборкой — против ручного прогона §38.4 (с оговорками выше).
- Побочный эффект изоляции: `LOCALAPPDATA` **не** перенаправляет DX-кэш драйвера NVIDIA — он живёт в общем `%LOCALAPPDATA%\NVIDIA` и подрос (~4.5 ГБ). Наш victory-кэш не тронут.

---

## 40. Раунд 17 — P0/P1 из `VIctory-NVIDIA`, сцен-матч A/B в 1080p и плечо D воды (27.11.2026)

Ветка `main-3060ti` (новых веток нет), 6 коммитов кода + доки; `victory-shader` не тронут, кэш `shaders-r8-group2` цел (27 180 файлов, последняя запись 14:29 — до прогонов). Гейты: `cargo check --workspace` — 0 ошибок; `cargo test --workspace -- --test-threads=1 --skip warp_budget_saturates_without_wraparound_and_preserves_draw_fields` — **684 passed, 0 failed** (`audit\round17\test-step1.log`).

### 40.1. Радар (AGENTS п.8; субагент упал, скан сделан вручную `gh`)

Скан 5 репозиториев 27.11: 60+ свежих PR. Отобрано (полностью — `audit\round17\radar.md`): Marvisak #3522 (census изменённого слота дешевле на ~25 % — прямо в наш `sync_heaps`), Belobug #622 (PSO в очередь при создании, а не при первом draw; прологовые стойки), 101Leafy #32 (`VICTORY_JOIN_ONE_CALL` — на 8-ГБ ноутбуке 58 `ExecuteCommandLists`/кадр), NVIDIA #48. **NVIDIA #45** (аллокация тайлов воды на wave32-хосте — ответ на тёмные плиты) — **NO-GO: `victory-shader`**; FrameGen #51 — NO-GO (RTX 30).

### 40.2. Порты P0/P1 (каждый — отдельный коммит, provenance в сообщении)

| коммит | что | дефолт |
|---|---|---|
| `848c45c` | `VICTORY_LEAVE_OUT`: дефрагментация decal-heap `02384a60fda6f2d0` + heat-haze `731a39171b568298` через наш `pipeline::left_out`; `=none`/список хэшей — откат | **вкл.** |
| `f69bb5a` | возврат резерва render-target'ов: `VICTORY_CARD_ROOM_FOR_TARGETS_SETTLE_MS` (20 000; 0 — как раньше), `targets_seen`/`targets_kept` | **вкл.** (20 с) |
| `937a6ce` | id хост-хипа при `release` динамических слотов (`47747d6`): устаревший список не освобождает слоты нового хипа | вкл. (корректность) |
| `f1bc2e9` | `VICTORY_WC_NO_BACK`: запоминание write-combined регионов (`0x400`) из XMemVirtualAlloc, забывание на remap; записи шейдеров в такие копии остаются на карте + строка `… left on the card` в кадре | **выкл.** |
| `310906a` | `VICTORY_HEAP_SYNC_ASIDE`/`_EARLY`: поток `victory-heap-sync`, ранние раунды каждые 4 мс, ожидание при drop guard'а перед отправкой списков; кадровая строка показывает ожидание | **вкл.** |
| `7ae4f57` | `VICTORY_LAND=1` (алиас `VICTORY_DELTA_LAND`) — наш батчевый dword-лендинг; `merge_piece` пропускает нетронутые 8-байтные блоки (эквивалентность byte-loop доказана тестом) | **выкл.** |

Не портировано (осознанно): пул потоков `land.rs` и нативная `safecopy` (AVX2-merge с отловом fault'ов на C/asm, ~1 200 строк) — в бюджет и риск раунда не влезли; `VICTORY_WC_BAR`; thread-ticket `854f57f` (`victory-shader`); helper-priority (`cores.rs` в нашем дереве нет).

### 40.3. Сцен-матч прогоны в 1080p (600 с; окно 430–600 с; лодочная миссия)

Профиль: `NATIVE_1440P=1`, `RENDER_SCALE=0.75`, `DLSS=0`, `FPS_COUNTER=all`, `WATCH_STATS=1`, `TILES_SMALL=512`; `VICTORY_KEYS`: Enter каждые 5 с до 300 с, `L` в 360/420/480/540. Тихий харнесс (без topmost и физ. ввода).

| прогон | профиль | fps p50 | ms p50 / p90 / p99 | фризы >150 мс | draws p50 | VRAM peak / mean |
|---|---|---|---|---|---|---|
| **база R15** (`r15-manual`, `RT_SHADOW=lit`) | — | **8,60** | 116,3 / 145,7 / 221,0 | 42 / 536 | 3 877 | 7 486 / 6 671 МБ |
| **1** (`road\r17-run1`) | P0 + `HEAP_SYNC_ASIDE` | **9,53** | 104,9 / 146,8 / 222,7 | 17 / 179 | 3 890 | 7 693 / 6 632 МБ |
| **2** (`road\r17-run2`) | 1 + `LAND` + `WC_NO_BACK` | 4,74 | 210,8 / 541,4 / 1 426,5 | 60 / 62 | 6 246 | 7 622 / 6 683 МБ |
| **3** (`road\r17-run3`) | 1 + `WATER_LIGHT=1` | 8,35 | 119,8 / 151,1 / 180,7 | 20 / 178 | 3 870 | 7 678 / 6 775 МБ |

**Равные корзины `draws`, база R15 → прогон 1** (та же сцена, `draws` совпал): 3700: 116,0→112,8 мс (1,03×); 3800: 112,1→100,8 (1,11×); **3900: 116,3→104,5 (1,11×)**; 4000: 117,7→110,5 (1,07×); 4200: 117,1→109,6 (1,07×). Итог: **+7…11 % FPS**, p99 без изменений (222,7 против 221,0), частота фризов та же (0,095 против 0,078 на сэмпл) — выигрыш в медиане, не в хвосте.

**Стадии в окне (p50):** preparing 84,3 (база 85,3), copies 33,2 (36,1), textures 9,7 (10,5), GPU 12,5 (14,5), **copy-back 67,8 (55,7 — +12 мс, причина не разобрана**: либо SETTLE вернул резерв и пошла очередь текстур, либо поток heap-sync отнимает CPU у relay; это задача Раунда 18).

- **Прогон 2 не подтверждён**: маршрут ушёл дальше по миссии (draws 5 334–6 942), вода белая на стриминге, и `WC_NO_BACK` оставил на карте **56 ГБ/кадр** объявленных write-combined записей (29 регионов). Флаги остаются opt-in, нужен изолированный A/B.
- **Прогон 3**: цена tiled-света воды **≈14,9 мс/кадр** (104,9→119,8), на равных корзинах 121,6 против базы 116,3 (−4,5 %).
- Лог `HEAP_SYNC_ASIDE` в прогоне 1: ~233–246 изменённых страниц хипа/кадр, ~90–96 тыс. слотов просмотрено, ~1,3–1,6 тыс. зеркалировано — ожидание на кадровой строке малое.

### 40.4. Вода — плечо D: причина доказана числом

| кадр | левая вода | правая (блики) | небо |
|---|---|---|---|
| PS5 Pro (эталон) | **#628985** (98,137,133) | #BDCDBA | #BBC9D4 |
| R15 база, shot-465 | #3C556A (60,85,106) | #959490 | #CFD9E3 |
| R17 прогон 1, shot-465 | #30485B (48,72,91) | #5B5E63 | #B2BEC9 |
| **R17 прогон 3 (`WATER_LIGHT=1`) shot-450 / 465** | **#5C8882 (92,136,130) / #588983 (88,137,131)** | #9DBCA4 / #9CC1A6 | #D4D9DF |

С включённым tiled-светом воды цвет **совпал с PS5 Pro** (G 136–137 против 137, B 130–131 против 133; было 85/106). Значит тёмно-синяя вода — это ровно выброшенные `5dc3b25986622949`/`6f225b641c5b0708`, а **не** `AMBIENT_SPECULAR` (гипотеза §38.7 снята). Цена: ~15 мс/кадр и **тёмные плиты вернулись** — скрин `audit\round17\artifacts\r17-run3_waterlight_colour_correct_dark_blocks_435.png`; пользователь подтвердил на живом прогоне: «цвет стал правильным, но артефакты на воде снова». Убрать плиты можно только починив tile allocation на wave32-хосте — это NVIDIA #45, а он в `victory-shader`.

### 40.5. Закрепление (по результату A/B)

- `launch-game.ps1` и `run-road.ps1`: `VICTORY_LEAVE_OUT='02384a60fda6f2d0,731a39171b568298'`, `VICTORY_HEAP_SYNC_ASIDE=1`, `VICTORY_HEAP_SYNC_EARLY=4`; `WC_NO_BACK`/`LAND`/`DELTA_LAND`/`WATER_LIGHT` очищаются (арм ставит своё).
- Код-дефолты: LEAVE_OUT, HEAP_SYNC_ASIDE/EARLY, SETTLE, heap-id — включены; WC_NO_BACK и LAND — выключены.

### 40.6. Раунд 18 (план)

1. **Вода**: решение по `WATER_LIGHT` — (а) отдельная ветка «tile allocation на wave32» (NVIDIA #45, `victory-shader`, новая ревизия кэша + полный re-warm) или (б) оставить выключенным; без этого цвет не получить ценой без плит.
2. **Изолированный A/B** `LAND` отдельно и `WC_NO_BACK` отдельно (по 300 с) + разбор +12 мс copy-back в прогоне 1 (гипотезы: SETTLE/поток heap-sync).
3. **Радар**: Marvisak #3522, Belobug #622, 101Leafy `VICTORY_JOIN_ONE_CALL`; NVIDIA #48.
4. VRAM: SETTLE на пике не помог (7 693 против 7 486) — проверить на длинном прогоне с `VICTORY_CARD_ROOM_FOR_TARGETS_MB`.

### 40.7. Артефакты

- Радар: `audit\round17\radar.md`; логи гейтов: `test-p0.log`, `test-step1.log`.
- Прогоны: `road\r17-run1/2/3`; анализ — `audit\round17\analyze-run1*.log`, `analyze-r17-run2/3.log`; скрины — `audit\round17\artifacts\` (`r17-run1_*`, `r17-run3_waterlight_*` (цвет+плиты), `r17-run2_land_wc_whitewater_585.png`).
- Сравнения: `r16-compare.ps1` (равные корзины + окна) и `r16-frames.ps1` (стадии) из Раунда 16, использованы как есть.
