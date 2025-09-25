# mytoken (Anchor)

Коротко, что тут:
- Это базовый Anchor-проект (инициализирован `anchor init mytoken`).
- В `tests/token_program.ts` лежат мои тесты для задания: create_token_account / mint_tokens / transfer_tokens.

Как запускать у себя (в нормальной среде):
1) Поставить Solana CLI и Anchor CLI (гуглится, шаги стандартные).
2) В корне этого проекта:
   - `anchor build` (сгенерит IDL и типы)
   - `anchor test` (поднимет локальную сеть и запустит тесты)

Если тесты ругаются на IDL/Anchor.toml – значит не собрали проект. Сначала `anchor build`.

P.S. Комментарии в тесте писал простым языком, чтобы было понятно, что происходит на каждом шаге.