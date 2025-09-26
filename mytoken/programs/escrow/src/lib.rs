use anchor_lang::prelude::*;
use anchor_spl::associated_token::{self, AssociatedToken};
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};

// NOTE: временный ID, нужно будет заменить на сгенерированный публичный ключ программы
// для учебных тестов можно оставить заглушку, но лучше потом обновить.
declare_id!("Escrow1111111111111111111111111111111111111");

#[program]
pub mod escrow {
    use super::*;

    pub fn create_escrow(ctx: Context<CreateEscrow>, amount: u64) -> Result<()> {
        require!(amount > 0, EscrowError::InvalidAmount);
        let escrow = &mut ctx.accounts.escrow_account;
        escrow.sender = ctx.accounts.sender.key();
        escrow.receiver = ctx.accounts.receiver.key();
        escrow.mint = ctx.accounts.mint.key();
        escrow.amount = amount;
        escrow.is_completed = false;
        escrow.bump = *ctx.bumps.get("escrow_account").unwrap();
        Ok(())
    }

    pub fn deposit_tokens(ctx: Context<DepositTokens>) -> Result<()> {
        let escrow = &ctx.accounts.escrow_account;
        require!(!escrow.is_completed, EscrowError::AlreadyCompleted);
        require_keys_eq!(escrow.sender, ctx.accounts.sender.key(), EscrowError::NotSender);
        // Если уже что-то лежит — запретим повторный депозит
        require!(ctx.accounts.vault_token_account.amount == 0, EscrowError::AlreadyDeposited);

        let cpi_accounts = Transfer {
            from: ctx.accounts.sender_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.sender.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, escrow.amount)?;
        emit!(DepositedEvent { escrow: ctx.accounts.escrow_account.key(), amount: escrow.amount });
        Ok(())
    }

    pub fn release_tokens(ctx: Context<ReleaseTokens>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;
        require!(!escrow.is_completed, EscrowError::AlreadyCompleted);
        require_keys_eq!(escrow.receiver, ctx.accounts.receiver.key(), EscrowError::NotReceiver);
        require!(ctx.accounts.vault_token_account.amount == escrow.amount, EscrowError::VaultBalanceMismatch);

        // Подпись PDA (escrow_account) как authority для перевода из vault -> receiver
        let seeds: &[&[u8]] = &[b"escrow", escrow.sender.as_ref(), escrow.receiver.as_ref(), escrow.mint.as_ref(), &[escrow.bump]];
        let signer_seeds = &[seeds];
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.receiver_token_account.to_account_info(),
            authority: ctx.accounts.escrow_account.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token::transfer(cpi_ctx, escrow.amount)?;
        escrow.is_completed = true;
        emit!(ReleasedEvent { escrow: escrow.key(), amount: escrow.amount });
        Ok(())
    }

    pub fn cancel_escrow(ctx: Context<CancelEscrow>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;
        require!(!escrow.is_completed, EscrowError::AlreadyCompleted);
        require_keys_eq!(escrow.sender, ctx.accounts.sender.key(), EscrowError::NotSender);

        let vault_balance = ctx.accounts.vault_token_account.amount;
        if vault_balance > 0 {
            // Возврат токенов обратно отправителю
            let seeds: &[&[u8]] = &[b"escrow", escrow.sender.as_ref(), escrow.receiver.as_ref(), escrow.mint.as_ref(), &[escrow.bump]];
            let signer_seeds = &[seeds];
            let cpi_accounts = Transfer {
                from: ctx.accounts.vault_token_account.to_account_info(),
                to: ctx.accounts.sender_token_account.to_account_info(),
                authority: ctx.accounts.escrow_account.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            token::transfer(cpi_ctx, vault_balance)?;
        }
        escrow.is_completed = true;
        emit!(CancelledEvent { escrow: escrow.key(), refunded_amount: vault_balance });
        Ok(())
    }
}

// ------------------ Accounts ------------------
#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct CreateEscrow<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,
    /// CHECK: только читаем
    pub receiver: UncheckedAccount<'info>,
    /// CHECK: mint токена
    pub mint: UncheckedAccount<'info>,
    #[account(
        init,
        payer = sender,
        seeds = [b"escrow", sender.key().as_ref(), receiver.key().as_ref(), mint.key().as_ref()],
        bump,
        space = 8 + EscrowAccount::SIZE
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositTokens<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,
    /// CHECK: mint
    pub mint: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"escrow", escrow_account.sender.as_ref(), escrow_account.receiver.as_ref(), escrow_account.mint.as_ref()], bump = escrow_account.bump)]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut)]
    pub sender_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ReleaseTokens<'info> {
    #[account(mut)]
    pub receiver: Signer<'info>,
    /// CHECK: mint
    pub mint: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"escrow", escrow_account.sender.as_ref(), escrow_account.receiver.as_ref(), escrow_account.mint.as_ref()], bump = escrow_account.bump)]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut)]
    pub receiver_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelEscrow<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,
    /// CHECK: mint
    pub mint: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"escrow", escrow_account.sender.as_ref(), escrow_account.receiver.as_ref(), escrow_account.mint.as_ref()], bump = escrow_account.bump)]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut)]
    pub sender_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

// ------------------ State ------------------
#[account]
pub struct EscrowAccount {
    pub sender: Pubkey,
    pub receiver: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub is_completed: bool,
    pub bump: u8,
}
impl EscrowAccount {
    pub const SIZE: usize = 32 + 32 + 32 + 8 + 1 + 1; // поля + bump + padding(нет) => 106, округлим до 108? Anchor не требует выравнивания, оставим 106
}

// ------------------ Events ------------------
#[event]
pub struct DepositedEvent {
    pub escrow: Pubkey,
    pub amount: u64,
}

#[event]
pub struct ReleasedEvent {
    pub escrow: Pubkey,
    pub amount: u64,
}

#[event]
pub struct CancelledEvent {
    pub escrow: Pubkey,
    pub refunded_amount: u64,
}

// ------------------ Errors ------------------
#[error_code]
pub enum EscrowError {
    #[msg("Amount must be > 0")] InvalidAmount,
    #[msg("Escrow already completed")] AlreadyCompleted,
    #[msg("Only sender can perform this action")] NotSender,
    #[msg("Only receiver can perform this action")] NotReceiver,
    #[msg("Deposit already done")] AlreadyDeposited,
    #[msg("Vault balance mismatch")] VaultBalanceMismatch,
}
