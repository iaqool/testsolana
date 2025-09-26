# TODO / Идеи для дальнейшего улучшения

## Escrow Program
- [x] Реальный program id (ключ сгенерирован, обновлены declare_id! и Anchor.toml)
- [ ] Добавить nonce / счетчик в seeds для нескольких активных сделок между одинаковыми sender/receiver/mint
- [ ] Поле `deadline` (u64 unix timestamp) + проверка при release / cancel (просроченные можно отменять только sender)
- [ ] Частичный release: хранить `remaining` и разрешать `release_partial(amount)` пока remaining > 0
- [ ] Закрывать EscrowAccount после `release`/`cancel` (`close = sender`) для возврата лампортов
- [ ] Отдельное событие `EscrowCreatedEvent`
- [ ] Парсинг логов событий в тестах (а не просто успех транзакций)
- [ ] Проверка корректности mint в token accounts (сейчас полагаемся на SPL)

## Token Program
- [ ] Перегенерировать IDL и типы (убрать ручной discriminator burn_tokens)
- [ ] Добавить негативные тесты (mint без authority, transfer недостаточной суммы)
- [ ] Добавить событие для burn (BurnEvent)

## Инфраструктура
- [ ] GitHub Actions workflow: артефакт с логами валидатора
- [ ] Добавить lint step (cargo clippy / eslint) в CI
- [ ] Кэширование зависимостей Rust/Node в CI

## Документация
- [ ] Отдельный README раздел с примером разбора PDA адреса
- [ ] Диаграмма последовательности: create -> deposit -> release
- [ ] Раздел "Типичные ошибки" (порт занят, blockhash expired)

## Эксперименты (опционально)
- [ ] Перейти на token_interface для mint (актуально для SPL v7)
- [ ] Добавить benchmarking (примерно измерить latency transfer vs release)
- [ ] Поддержка расширенных разрешений (дополнительный arbiter)
