mod errors;
mod utils;

use put_anchor_lang::prelude::*;
use put_anchor_lang::put_program::system_instruction;
use put_anchor_lang::put_program::system_program;
use put_anchor_ppl::token::{
    ppl_token, ppl_token::instruction as token_instruction, Mint, Token, TokenAccount,
};

use errors::ReelPayError;

declare_id!("paytGwzjKgffkpCPPTzMbKJV1miozAjuXpzjZx6it5T");

const NATIVE_SEED: &[u8] = b"put_pool";
const ADMIN_SEED: &[u8] = b"admin";

#[program]
pub mod reelpay {

    use super::*;

    pub fn pay_native(ctx: Context<PayAccounts>, args: PayARGS) -> Result<()> {
        require!(
            ctx.accounts.to_account.owner.to_string() == crate::ID.to_string(),
            ReelPayError::Forbidden
        );
        let transfer_instruction = system_instruction::transfer(
            ctx.accounts.from_account.key,
            ctx.accounts.to_account.key,
            args.amount,
        );
        // 主币转账
        put_anchor_lang::put_program::program::invoke(
            &transfer_instruction,
            &[
                ctx.accounts.from_account.to_account_info(),
                ctx.accounts.to_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        utils::log(format!("{},{}", args.order_id, args.amount));

        Ok(())
    }

    pub fn pay_token(
        ctx: Context<PayTokenAccounts>,
        _token_pool_seed: String,
        args: PayARGS,
    ) -> Result<()> {
        // require!(ctx.accounts.to_account.amount, ReelPayError::ArgsError);

        let transfer_instruction = token_instruction::transfer(
            ctx.accounts.token_program.key,
            &ctx.accounts.from_account.key(),
            &ctx.accounts.to_account.key(),
            ctx.accounts.payer.key,
            &[],
            args.amount,
        )?;
        put_anchor_lang::put_program::program::invoke(
            &transfer_instruction,
            &[
                ctx.accounts.from_account.to_account_info(),
                ctx.accounts.to_account.to_account_info(),
                ctx.accounts.payer.to_account_info(),
            ],
        )?;
        utils::log(format!(
            "{},{},{}",
            args.order_id,
            args.amount,
            ctx.accounts.mint_account.key().to_string()
        ));
        Ok(())
    }

    pub fn withdraw(ctx: Context<WithdrawAccounts>, args: WithdrawARGS) -> Result<()> {
        require!(
            ctx.accounts.admin.0 == ctx.accounts.payer.key(),
            ReelPayError::Forbidden
        );

        if args.pay_is_main {
            let (put_pool, _bump) =
                Pubkey::find_program_address(&[args.seed.as_bytes()], ctx.program_id);
            require!(
                ctx.accounts.from_account.key.to_string() == put_pool.key().to_string(),
                ReelPayError::Forbidden
            );
            let from_amount = ctx.accounts.from_account.lamports();
            let to_amount = ctx.accounts.to_account.lamports();
            let final_from_amount = from_amount.checked_sub(args.amount);
            let final_to_amount = to_amount.checked_add(args.amount);
            if final_from_amount.is_some() && final_to_amount.is_some() {
                **ctx
                    .accounts
                    .from_account
                    .to_account_info()
                    .lamports
                    .borrow_mut() = final_from_amount.unwrap();
                **ctx
                    .accounts
                    .to_account
                    .to_account_info()
                    .lamports
                    .borrow_mut() = final_to_amount.unwrap()
            }
        } else {
            let (token_pool, bump) =
                Pubkey::find_program_address(&[args.seed.as_bytes()], ctx.program_id);
            require!(
                token_pool.to_string() == ctx.accounts.from_account.key().to_string(),
                ReelPayError::Forbidden
            );

            let transfer_instruction = ppl_token::instruction::transfer(
                ctx.accounts.token_program.key,
                ctx.accounts.from_account.key,
                ctx.accounts.to_account.key,
                ctx.accounts.from_account.key,
                &[ctx.accounts.from_account.key],
                args.amount,
            )?;
            put_anchor_lang::put_program::program::invoke_signed(
                &transfer_instruction,
                &[
                    ctx.accounts.token_program.to_account_info(),
                    ctx.accounts.from_account.to_account_info(),
                    ctx.accounts.to_account.to_account_info(),
                    ctx.accounts.from_account.to_account_info(),
                ],
                &[&[args.seed.as_bytes(), &[bump]]],
            )?;
        }
        Ok(())
    }

    pub fn change_admin(ctx: Context<ChangeAdmin>, args: ChangeAdminARGS) -> Result<()> {
        require!(
            ctx.accounts.admin.0.to_string() == ctx.accounts.payer.key().to_string(),
            ReelPayError::Forbidden
        );
        ctx.accounts.admin.change_admin(args.address);

        Ok(())
    }

    // 初始化关联账户
    pub fn initialize(ctx: Context<Initialize>, args: InitializeARGS) -> Result<()> {
        require!(
            ctx.accounts.admin.0.to_string() == system_program::ID.to_string(),
            ReelPayError::Forbidden
        );
        ctx.accounts.admin.change_admin(args.admin);
        Ok(())
    }
    // 创建池子
    pub fn create_pool(ctx: Context<CreatePoolAccounts>, seed: String) -> Result<()> {
        require!(
            ctx.accounts.admin.0 == ctx.accounts.payer.key(),
            ReelPayError::Forbidden
        );
        let (pool, _bump) = Pubkey::find_program_address(&[seed.as_bytes()], ctx.program_id);
        require!(
            pool.to_string() == ctx.accounts.token_pool.key().to_string(),
            ReelPayError::Forbidden
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init_if_needed, payer = payer, space = 8 + 32, seeds = [ADMIN_SEED], bump)]
    admin: Account<'info, Admin>,
    /// CHECK:
    #[account(init_if_needed, space = 8, payer = payer, seeds = [NATIVE_SEED], bump)]
    put_pool: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PayAccounts<'info> {
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    #[account(mut)]
    payer: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    from_account: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, seeds=[NATIVE_SEED], bump)]
    to_account: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(token_pool_seed: String)]
pub struct PayTokenAccounts<'info> {
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    mint_account: Account<'info, Mint>,
    #[account(mut)]
    payer: Signer<'info>,
    /// CHECK:
    #[account(mut, token::mint = mint_account)]
    from_account: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(mut, token::mint = mint_account, token::authority = to_account, seeds = [token_pool_seed.as_bytes()], bump)]
    to_account: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct WithdrawAccounts<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    /// CHECK:
    #[account(address = crate::ID)]
    reelpay_program: AccountInfo<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    /// CHECK:
    #[account(mut)]
    from_account: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    to_account: AccountInfo<'info>,
    #[account(seeds = [ADMIN_SEED], bump)]
    admin: Account<'info, Admin>,
}

#[derive(Accounts)]
pub struct ChangeAdmin<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    system_program: Program<'info, System>,
    #[account(seeds = [ADMIN_SEED], bump)]
    admin: Account<'info, Admin>,
}

#[derive(Accounts)]
#[instruction(token_pool_seed: String)]
pub struct CreatePoolAccounts<'info> {
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [ADMIN_SEED], bump)]
    admin: Account<'info, Admin>,
    pub token_mint: Account<'info, Mint>,
    #[account(init_if_needed, payer = payer, token::mint = token_mint, token::authority = token_pool, seeds = [token_pool_seed.as_bytes()], bump)]
    pub token_pool: Account<'info, TokenAccount>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreatePoolARGS {
    seed: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct PayARGS {
    order_id: String,
    amount: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawARGS {
    pay_is_main: bool,
    amount: u128,
    seed: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeARGS {
    admin: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ChangeAdminARGS {
    address: Pubkey,
}

#[account]
pub struct Admin(Pubkey);

impl Admin {
    pub fn change_admin(&mut self, new_admin: Pubkey) {
        self.0 = new_admin;
    }
}
