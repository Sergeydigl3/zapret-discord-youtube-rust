# Changelog

## [2.2.0](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/compare/zapret-rust-v2.1.0...zapret-rust-v2.2.0) (2026-09-30)


### 🚀 Новые функции

* add mouse support ([430cc58](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/430cc582be09e68ec9523702018b7d5c53d2c4b5))
* add router mode ([370ff4c](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/370ff4c4e7cde112b0b67871e9ba26649850588e))


### 🐛 Исправления ошибок

* **autotune:** resolve strategies through strategy::resolve ([c6aeeaa](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/c6aeeaadd028327bb070855012c0ee9676e15c8f))
* **config:** исправить дефолтный фейк GameFilter UDP ([62d2f25](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/62d2f254bc9cc88bd0961ef4a41fbbaff7c5958a))
* **core:** hide deps under platform selector ([40c4a8b](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/40c4a8b5b5d41ed1141c0fea76f46d2fbfddc3e0))
* ttl flags works ([067da2a](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/067da2ab0a3e4ca439e7326eb07253a0a381eb70))
* **windows:** TUI в Windows Terminal, цветные эмодзи и UTF-8 ([76540dc](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/76540dcc38ffde5b7ef6a1526ac5cbdd3265b963))


### ♻️ Рефакторинг кода

* **autotune:** move the probing and check modules under autotune ([15a929e](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/15a929e7a680ddd1f0ee6730791416e11fe4b5fb))
* **core:** extract the downloader into zapret-fetch ([30470c7](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/30470c733a8c4820e9f544290d5c893c5ea88215))
* **core:** make the runner take a LaunchPlan instead of a strategy name ([872af09](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/872af092a0eca4cff6e58ac568555b6723ff7e35))
* **core:** process management handling ([bd74a01](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/bd74a01cd0d2f645dfd7370f426aa73aa570292b))
* move the framing into zapret-wrapper ([829b57b](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/829b57b486c51c424d13a870542031cd8c28f16f))
* split the codebase into zapret-core and zapret-tui crates ([3cae5cb](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/3cae5cb772bf66516573b18a7b1ad3f8e2c5ba14))
* use native uid check ([c789650](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/c7896502054ce0717c260b7fce5d6f494cd57f74))


### 📝 Документация

* add NixOS module usage documentation to README ([bde98cd](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/bde98cdb385f132aaf716dbddf0542ae059e917f))
* rewrite the package map for the five-crate split ([e87a5e7](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/e87a5e73b1a610fe46e1d9f553a15da5c834df09))


### 🔧 Обслуживание и зависимости

* add background service support for NixOS ([0847893](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/084789359e8967a6c0e1397c9aa74efc1510f0b2))
* tmp remove later ([670a3cc](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/670a3ccf7b29e9d2139288af5599ae56705a3c9e))

## [2.1.0](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/compare/zapret-rust-v2.0.0...zapret-rust-v2.1.0) (2026-09-01)


### 🚀 Новые функции

* **strategy:** добавить кастомные стратегии ([78d9a3a](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/78d9a3ac5d4200628bfc6b16d72b42bb7553d8f3))


### 🐛 Исправления ошибок

* **gamefilter:** Фикс заменение портов геймфильтра на неправильные ([6cff785](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/6cff78517a44e8059d11d0b0a618e99cb3c8838f))
* **tui:** исправление неотзывчивого ввода в nano при редактировании листов ([5915109](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/59151098f942d574d6018a49b4a5bb0eef7dde18))


### 🔧 Обслуживание и зависимости

* **custom-strategies:** обновление кастомных стратегий под страндарт flowseal ([af40a34](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/af40a342da7312c1729a36d1a228c8766b24358e))

## [2.0.0](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/compare/zapret-rust-v1.1.0...zapret-rust-v2.0.0) (2026-08-10)


### ⚠ BREAKING CHANGES

* add basic CI for release and PR checks

### 🚀 Новые функции

* add basic CI for release and PR checks ([f54a7c6](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/f54a7c6ef6a0a402fff625ed90d7a80577897a30))
* **autotune:** автоматически сбрасывать TTL в auto перед проверкой и восстанавливать пользовательский TTL ([7f3dc59](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/7f3dc59dafdd3186b18569bfc33209bfd88c8eb1))
* **autotune:** добавить возможность экстренной остановки автотюнинга по клавишам 'q' и Esc ([9e254ae](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/9e254aea7b62bf7e463eb33a364508f1501f776e))
* **tui:** add vim motion keys for menu navigation ([#35](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/issues/35)) ([2aaae4b](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/2aaae4bc7080e1f3cb3a99b0c1414cb9c04c6f0c))
* автотюн на реальных QUIC-пакетах, подбор DPI TTL и редактируемые списки доменов ([c44d5a4](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/c44d5a4bdfed81c5db44feba909309e956a2113c))


### 🐛 Исправления ошибок

* **autotune:** исправить точный расчёт прогресса тестирования стратегий ([93b9aa1](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/93b9aa1dd8bb486a74da6e77b5186bd7d4b2eefe))
* **autotune:** расчитывать общее число запросов заранее и добавить таймер выполнения ([c87feca](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/c87fecade62e813bbd341eb14086e5ad590d8c0f))
* **autotune:** сохранять и выводить итоговое время выполнения в отчёт результатов ([dd01d0a](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/dd01d0a126228f56eabb0aeb5e41932d2a417ffc))
* **runner:** убрать вызов setcap на Windows ([6f09b14](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/6f09b14c9077f0179b5b4bccc09b9e94b57028a8))
* фиксированный TTL переопределяет параметры ttl/autottl из стратегии ([df99b16](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/df99b169993099647b9b81d5ebb5e4492116b6eb))


### ⚡ Улучшения производительности

* **autotune:** добавить мгновенное прерывание процессов curl и подпотоков при нажатии q/Esc ([e1c9065](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/e1c9065fdbd40933c1c70091df1d244010e7727d))
* **autotune:** заменить вызовы внешнего curl на нативные сетевые запросы ureq ([ef72523](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/ef72523b2b5b8796a2ec4d5eb77b63912e0fc2a6))
* **autotune:** использовать scoped threads в std::thread::scope и наладить максимальный профиль сборки release ([803164b](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/803164bdd9c318a462e9feac1d2099924643de60))
* **strategy:** использовать OnceLock для кэширования регулярных выражений и уменьшить задержки старта процессов ([a512591](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/a5125913ee690ef114cf2ab36d652bce7b15e19e))
* интегрировать DNS-кэш в TCP-подключения, включить пул соединений HTTP и максимальные настройки релизного профиля ([8ed9093](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/8ed9093f47b4d6e050c829767af0970a9aa49aa7))


### ♻️ Рефакторинг кода

* **autotune:** разбить монолитный модуль на подмодули и добавить кэширование DNS ([b6cb1d6](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/b6cb1d66fd528572157f8adec02f456782b5db32))


### 📝 Документация

* document nix flake installation and dev-shell ([1dd4685](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/1dd46856f78d7cd940f963310b8770bb73774173))
* обновить README под новые возможности ([c822bea](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/c822bea485edc9e0e0d558cba358c835af7fd205))
* обновить README с информацией об экстренной остановке по q/Esc, авто-сбросе TTL и таймере ([4cbb0b8](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/4cbb0b8c6a211e4cc2385440bee8fa3f0ef3984f))


### 🔧 Обслуживание и зависимости

* add nix flake ([10cde3f](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/10cde3fae60b43140bd194d2bd5534a29bf57b33))
* enable check-style on PR ([48d96aa](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/48d96aa82055253975f120c3d6bb46a79492dff4))
* **master:** release zapret-rust 1.0.0 ([25a8b63](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/25a8b63a24088fb57ad5ee94f35ee546595125e0))
* **master:** release zapret-rust 1.1.0 ([f44188d](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/f44188d1b101a582f2ed951a163cf0aa6abbb05d))

## [1.1.0](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/compare/zapret-rust-v1.0.0...zapret-rust-v1.1.0) (2026-08-09)


### 🚀 Новые функции

* **autotune:** автоматически сбрасывать TTL в auto перед проверкой и восстанавливать пользовательский TTL ([7f3dc59](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/7f3dc59dafdd3186b18569bfc33209bfd88c8eb1))
* **autotune:** добавить возможность экстренной остановки автотюнинга по клавишам 'q' и Esc ([9e254ae](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/9e254aea7b62bf7e463eb33a364508f1501f776e))
* **tui:** add vim motion keys for menu navigation ([#35](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/issues/35)) ([2aaae4b](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/2aaae4bc7080e1f3cb3a99b0c1414cb9c04c6f0c))
* автотюн на реальных QUIC-пакетах, подбор DPI TTL и редактируемые списки доменов ([c44d5a4](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/c44d5a4bdfed81c5db44feba909309e956a2113c))


### 🐛 Исправления ошибок

* **autotune:** исправить точный расчёт прогресса тестирования стратегий ([93b9aa1](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/93b9aa1dd8bb486a74da6e77b5186bd7d4b2eefe))
* **autotune:** расчитывать общее число запросов заранее и добавить таймер выполнения ([c87feca](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/c87fecade62e813bbd341eb14086e5ad590d8c0f))
* **autotune:** сохранять и выводить итоговое время выполнения в отчёт результатов ([dd01d0a](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/dd01d0a126228f56eabb0aeb5e41932d2a417ffc))
* **runner:** убрать вызов setcap на Windows ([6f09b14](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/6f09b14c9077f0179b5b4bccc09b9e94b57028a8))
* фиксированный TTL переопределяет параметры ttl/autottl из стратегии ([df99b16](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/df99b169993099647b9b81d5ebb5e4492116b6eb))


### ⚡ Улучшения производительности

* **autotune:** добавить мгновенное прерывание процессов curl и подпотоков при нажатии q/Esc ([e1c9065](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/e1c9065fdbd40933c1c70091df1d244010e7727d))
* **autotune:** заменить вызовы внешнего curl на нативные сетевые запросы ureq ([ef72523](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/ef72523b2b5b8796a2ec4d5eb77b63912e0fc2a6))
* **autotune:** использовать scoped threads в std::thread::scope и наладить максимальный профиль сборки release ([803164b](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/803164bdd9c318a462e9feac1d2099924643de60))
* **strategy:** использовать OnceLock для кэширования регулярных выражений и уменьшить задержки старта процессов ([a512591](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/a5125913ee690ef114cf2ab36d652bce7b15e19e))
* интегрировать DNS-кэш в TCP-подключения, включить пул соединений HTTP и максимальные настройки релизного профиля ([8ed9093](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/8ed9093f47b4d6e050c829767af0970a9aa49aa7))


### ♻️ Рефакторинг кода

* **autotune:** разбить монолитный модуль на подмодули и добавить кэширование DNS ([b6cb1d6](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/b6cb1d66fd528572157f8adec02f456782b5db32))


### 📝 Документация

* document nix flake installation and dev-shell ([1dd4685](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/1dd46856f78d7cd940f963310b8770bb73774173))
* обновить README под новые возможности ([c822bea](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/c822bea485edc9e0e0d558cba358c835af7fd205))
* обновить README с информацией об экстренной остановке по q/Esc, авто-сбросе TTL и таймере ([4cbb0b8](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/4cbb0b8c6a211e4cc2385440bee8fa3f0ef3984f))


### 🔧 Обслуживание и зависимости

* add nix flake ([10cde3f](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/10cde3fae60b43140bd194d2bd5534a29bf57b33))
* enable check-style on PR ([48d96aa](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/48d96aa82055253975f120c3d6bb46a79492dff4))

## [1.0.0](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/compare/zapret-rust-v0.1.0...zapret-rust-v1.0.0) (2026-07-30)


### ⚠ BREAKING CHANGES

* add basic CI for release and PR checks

### 🚀 Новые функции

* add basic CI for release and PR checks ([f54a7c6](https://github.com/Sergeydigl3/zapret-discord-youtube-rust/commit/f54a7c6ef6a0a402fff625ed90d7a80577897a30))
