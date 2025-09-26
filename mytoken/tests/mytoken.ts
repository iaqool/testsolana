import * as anchor from "@coral-xyz/anchor";
import fs from "fs";
import path from "path";

// Простой тест-шаблон от Anchor. Я добавил проверку окружения,
// чтобы в этой демо-среде он просто скипался, а у себя вы могли запустить.

const hasAnchorToml = fs.existsSync(path.resolve(process.cwd(), "Anchor.toml"));

if (!hasAnchorToml) {
  describe("mytoken (skipped)", () => {
    it("skips because no Anchor workspace/IDL here", function () {
      this.skip();
    });
  });
} else {
  describe("mytoken", () => {
    // Подключаемся: если нет ANCHOR_PROVIDER_URL, используем localhost
    if (!process.env.ANCHOR_PROVIDER_URL) {
      process.env.ANCHOR_PROVIDER_URL = 'http://127.0.0.1:8899';
    }
    if (!process.env.ANCHOR_WALLET) {
      process.env.ANCHOR_WALLET = path.resolve(process.env.HOME || '~', '.config/solana/id.json');
    }
    anchor.setProvider(anchor.AnchorProvider.env());

    // В реальном проекте тут обычно используют типы из IDL,
    // но чтобы не зависеть от сборки — оставлю any.
    const program = (anchor.workspace as any).Mytoken as any;

    it("Is initialized!", async () => {
      // Простейший вызов initialize, чисто sanity-check
      const tx = await program.methods.initialize().rpc();
      console.log("tx:", tx);
    });
  });
}
