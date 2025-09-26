use anchor_lang::prelude::*;
use anchor_spl::associated_token::{self, AssociatedToken};
use anchor_spl::token::{self, Mint, MintTo, Token, Transfer, Burn};
use anchor_lang::solana_program::{program::invoke, system_instruction};

declare_id!("EYfHSdmUTkcXEt2rsUUdW16C9taGPR8sXjMsQqV4F5pZ");

#[program]
pub mod mytoken {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    // Создание нового токена (mint) с decimals=0 и authority = authority
    pub fn create_token(ctx: Context<CreateToken>) -> Result<()> {
        let authority = &ctx.accounts.authority;
        let mint = &ctx.accounts.mint;
        let system_program = &ctx.accounts.system_program;
        let token_program = &ctx.accounts.token_program;

        // Создаём аккаунт под Mint вручную
        let lamports = Rent::get()?.minimum_balance(Mint::LEN);
        let create_ix = system_instruction::create_account(
            authority.key,
            mint.key,
            lamports,
            Mint::LEN as u64,
            &token_program.key(),
        );
        invoke(
            &create_ix,
            &[
                authority.to_account_info(),
                mint.to_account_info(),
                system_program.to_account_info(),
            ],
        )?;

        // Инициализируем mint (decimals=0, authority = authority)
        let cpi_accounts = token::InitializeMint2 {
            mint: mint.to_account_info(),
        };
        token::initialize_mint2(
            CpiContext::new(token_program.to_account_info(), cpi_accounts),
            0,
            authority.key,
            Some(&authority.key()),
        )?;

        Ok(())
    }

    // Создание ATA (associated token account) для (authority, mint)
    pub fn create_token_account(ctx: Context<CreateTokenAccount>) -> Result<()> {
        let cpi_program = ctx.accounts.associated_token_program.to_account_info();
        let cpi_accounts = associated_token::Create {
            payer: ctx.accounts.authority.to_account_info(),
            associated_token: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };

        associated_token::create(CpiContext::new(cpi_program, cpi_accounts))
    }

    // Mint токенов на указанный token_account
    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        token::mint_to(CpiContext::new(cpi_program, cpi_accounts), amount)
    }

    // Перевод токенов между счетами
    pub fn transfer_tokens(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        token::transfer(CpiContext::new(cpi_program, cpi_accounts), amount)
    }

    // Сжигание токенов с указанного token_account, уменьшая общий supply
    pub fn burn_tokens(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
        // CPI к SPL Token: Burn
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        token::burn(CpiContext::new(cpi_program, cpi_accounts), amount)
    }
}

#[derive(Accounts)]
pub struct Initialize {}

// Accounts для create_token: создаём новый Mint с authority и decimals=0
#[derive(Accounts)]
pub struct CreateToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: создаём вручную как Mint
    #[account(mut, signer)]
    pub mint: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// Accounts для создания ATA: создаём associated token account для пары (authority, mint)
#[derive(Accounts)]
pub struct CreateTokenAccount<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: mint проверяется внутри SPL CPI
    pub mint: UncheckedAccount<'info>,
    /// CHECK: создаётся программой associated token
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// Accounts для mint_tokens
#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    /// CHECK: mint
    pub mint: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: token account
    pub token_account: UncheckedAccount<'info>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// Accounts для transfer_tokens
#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    /// CHECK: from token account
    pub from: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: to token account
    pub to: UncheckedAccount<'info>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// Accounts для burn_tokens
#[derive(Accounts)]
pub struct BurnTokens<'info> {
    #[account(mut)]
    /// CHECK: mint
    pub mint: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: token account (holder)
    pub token_account: UncheckedAccount<'info>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}
