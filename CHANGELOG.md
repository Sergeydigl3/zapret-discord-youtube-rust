# Changelog

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
