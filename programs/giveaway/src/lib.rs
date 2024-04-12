mod errors;
mod event_type;
mod utils;

use put_anchor_lang::prelude::*;
use put_anchor_lang::put_program::clock::Clock;
use put_anchor_lang::put_program::secp256k1_recover::secp256k1_recover;
use put_anchor_ppl::token::{
    ppl_token, ppl_token::instruction as token_instruction, Mint, Token, TokenAccount,
};
use sha3::{Digest, Keccak256};

use errors::GiveawayError;

declare_id!("3Lkno95uimuGtwLXv3oNBhqraJmiaeFMiDakDfo449R4");

#[program]
pub mod giveaway {

    use put_anchor_lang::put_program::program::invoke;

    use super::*;

    pub fn create_put_giveaway(
        ctx: Context<CreatePutGiveawayAccounts>,
        args: CreateGiveawayARG,
    ) -> Result<()> {
        ctx.accounts.giveaway_pool.creator = ctx.accounts.payer.key();
        ctx.accounts.giveaway_pool.receive_records = Vec::new();
        ctx.accounts.giveaway_pool.total_amount = args.amount;

        let transfer_instruction = put_anchor_lang::put_program::system_instruction::transfer(
            ctx.accounts.payer.key,
            &ctx.accounts.giveaway_pool.key(),
            args.amount,
        );
        // 主币转账
        put_anchor_lang::put_program::program::invoke(
            &transfer_instruction,
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.giveaway_pool.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        let giveaway_id_str = hex::encode(args.giveaway_id.to_vec());

        let log_msg = format!(
            "{},{},{},{},{}",
            event_type::EventType::Create as u32,
            giveaway_id_str,
            args.amount,
            ctx.accounts.payer.key.to_string(),
            "0",
        );

        utils::log(log_msg);
        Ok(())
    }

    pub fn create_non_put_giveaway(
        ctx: Context<CreateNonPutGiveawayAccounts>,
        args: CreateGiveawayARG,
    ) -> Result<()> {
        ctx.accounts.giveaway_pool.creator = ctx.accounts.payer.key();
        ctx.accounts.giveaway_pool.receive_records = Vec::new();
        ctx.accounts.giveaway_pool.total_amount = args.amount;

        let token_tx = token_instruction::transfer(
            ctx.accounts.token_program.key,
            &ctx.accounts.from_account.key(),
            &ctx.accounts.token_pool.key(),
            ctx.accounts.payer.key,
            &[],
            args.amount,
        )?;

        invoke(
            &token_tx,
            &[
                ctx.accounts.from_account.to_account_info(),
                ctx.accounts.token_pool.to_account_info(),
                ctx.accounts.payer.to_account_info(),
            ],
        )?;

        let giveaway_id_str = hex::encode(args.giveaway_id.to_vec());

        let log_msg = format!(
            "{},{},{},{},{}",
            event_type::EventType::Create as u32,
            giveaway_id_str,
            args.amount,
            ctx.accounts.payer.key.to_string(),
            ctx.accounts.token_mint.key().to_string(),
        );

        utils::log(log_msg);

        Ok(())
    }

    pub fn receive_put_giveaway(
        ctx: Context<ReceivePutGiveawayAccount>,
        args: ReceivePutGiveawayARG,
    ) -> Result<()> {
        let timestamp_bytes = args.timestamp.to_be_bytes();
        let amount_bytes = args.amount.to_be_bytes();
        let giveaway_id_bytes = args.giveaway_id;
        let wallet_address_bytes = args.wallet_address;

        // let amount_non_zero_position = amount_bytes.iter().position(|&x| x != 0).unwrap();
        // let timestamp_non_zero_position = timestamp_bytes.iter().position(|&x| x != 0).unwrap();

        let (pool, _bump) =
            Pubkey::find_program_address(&[&args.giveaway_id.clone()], ctx.program_id);

        // 检查时效性
        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp.unsigned_abs();

        require!(args.timestamp > current_timestamp, GiveawayError::Overtime);

        // 检查余额是否充足
        require!(
            args.amount <= ctx.accounts.giveaway_pool.total_amount,
            GiveawayError::ExceedError
        );

        // 校验签名正确
        let origin_message_bytes = [
            wallet_address_bytes.as_ref(),
            giveaway_id_bytes.as_ref(),
            timestamp_bytes.as_ref(),
            amount_bytes.as_ref(),
        ]
        .concat();

        // let prefix_str = String::from("\x19Ethereum Signed Message:\n");
        // let prefix_bytes = prefix_str.as_bytes();
        // let message_len = origin_message_bytes.len().to_string(); // 必须是字符串len
        // let message_bytes = message_len.as_bytes();
        // let message = [prefix_bytes, message_bytes, origin_message_bytes.as_slice()].concat();

        let mut hasher_sig = Keccak256::new();
        hasher_sig.update(origin_message_bytes.as_slice());
        let sig_message = hasher_sig.finalize();

        let signature = &args.signature[0..64];
        let recover_id = args.signature[64] - 27;

        let secp_pub = secp256k1_recover(&sig_message, recover_id, &signature).unwrap();

        let mut hasher_pub = Keccak256::new();
        hasher_pub.update(secp_pub.to_bytes());
        let signed_pub = hasher_pub.finalize();

        let pub_key = &signed_pub[12..];

        require!(
            pub_key.eq(giveaway_id_bytes.as_ref()),
            GiveawayError::Forbidden
        );

        let from_account = ctx.accounts.giveaway_pool.to_account_info().lamports();
        let to_account = ctx.accounts.payer.to_account_info().lamports();

        let final_from_amount = from_account.checked_sub(args.amount);
        let final_to_amount = to_account.checked_add(args.amount);

        if final_from_amount.is_some() && final_to_amount.is_some() {
            **ctx
                .accounts
                .giveaway_pool
                .to_account_info()
                .lamports
                .borrow_mut() = final_from_amount.unwrap();

            **ctx.accounts.payer.to_account_info().lamports.borrow_mut() = final_to_amount.unwrap();
        }

        let giveaway_id_str = hex::encode(args.giveaway_id.to_vec());

        let log_msg = format!(
            "{},{},{},{},{}",
            event_type::EventType::Receive as u32,
            giveaway_id_str,
            args.amount,
            ctx.accounts.payer.key.to_string(),
            "0",
        );

        utils::log(log_msg);

        Ok(())
    }

    pub fn receive_non_put_giveaway(
        ctx: Context<ReceiveNonPutGiveawayAccount>,
        args: ReceiveNonPutGiveawayARG,
    ) -> Result<()> {
        let timestamp_bytes = args.timestamp.to_be_bytes();
        let amount_bytes = args.amount.to_be_bytes();
        let giveaway_id_bytes = args.giveaway_id;
        let to_account_bytes = ctx.accounts.to_account.key().to_bytes();
        let mint_address = ctx.accounts.token_mint.key().to_bytes();

        // let amount_non_zero_position = amount_bytes.iter().position(|&x| x != 0).unwrap();
        // let timestamp_non_zero_position = timestamp_bytes.iter().position(|&x| x != 0).unwrap();

        let (_pool, _bump) = Pubkey::find_program_address(
            &[
                &ctx.accounts.payer.key().to_bytes(),
                &ctx.accounts.token_mint.key().to_bytes(),
            ],
            ctx.program_id,
        );

        // 检查时效性
        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp.unsigned_abs();
        require!(args.timestamp > current_timestamp, GiveawayError::Overtime);

        // 检查余额是否充足
        require!(
            args.amount <= ctx.accounts.giveaway_pool.total_amount,
            GiveawayError::ExceedError
        );

        // 校验签名正确
        let origin_message_bytes = [
            ctx.accounts.payer.key().as_ref(),
            giveaway_id_bytes.as_ref(),
            timestamp_bytes.as_ref(),
            amount_bytes.as_ref(),
        ]
        .concat();

        let mut hash_origin_message: sha3::digest::core_api::CoreWrapper<sha3::Keccak256Core> =
            Keccak256::new();
        hash_origin_message.update(origin_message_bytes.as_slice());
        let hashed_origin_message = hash_origin_message.finalize();

        let signature = &args.signature[0..64];
        let recover_id = args.signature[64] - 27;

        let secp_pub =
            secp256k1_recover(hashed_origin_message.as_slice(), recover_id, &signature).unwrap();
        secp_pub.to_bytes();

        let mut hasher_pub = Keccak256::new();
        hasher_pub.update(secp_pub.to_bytes());
        let signed_pub = hasher_pub.finalize();
        let pub_key = &signed_pub[12..];

        require!(
            pub_key.eq(giveaway_id_bytes.as_ref()),
            GiveawayError::Forbidden
        );

        let transfer_instruction = ppl_token::instruction::transfer(
            ctx.accounts.token_program.key,
            &ctx.accounts.token_pool.key(),
            &ctx.accounts.to_account.key(),
            &ctx.accounts.token_pool.key(),
            &[&ctx.accounts.token_pool.key()],
            args.amount,
        )?;
        put_anchor_lang::put_program::program::invoke_signed(
            &transfer_instruction,
            &[
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.token_pool.to_account_info(),
                ctx.accounts.to_account.to_account_info(),
            ],
            &[&[
                ctx.accounts.payer.key.as_ref(),
                ctx.accounts.token_mint.key().as_ref(),
                &[_bump],
            ]],
        )?;

        let giveaway_id_str = hex::encode(args.giveaway_id.to_vec());

        let log_msg = format!(
            "{},{},{},{},{}",
            event_type::EventType::Receive as u32,
            giveaway_id_str,
            args.amount,
            ctx.accounts.payer.key.to_string(),
            ctx.accounts.token_mint.key().to_string(),
        );

        utils::log(log_msg);
        Ok(())
    }

    pub fn refund_put(
        ctx: Context<RefundPutGiveawayAccount>,
        args: RefundPutGiveawayARGS,
    ) -> Result<()> {
        require!(
            ctx.accounts
                .payer
                .key()
                .as_ref()
                .eq(ctx.accounts.giveaway_pool.creator.key().as_ref()),
            GiveawayError::Forbidden
        );

        let remain_fund = ctx.accounts.giveaway_pool.total_amount;
        ctx.accounts.giveaway_pool.total_amount = 0;

        let from_account = ctx.accounts.giveaway_pool.to_account_info().lamports();
        let to_account = ctx.accounts.payer.to_account_info().lamports();

        let final_from_amount = from_account.checked_sub(remain_fund);
        let final_to_amount = to_account.checked_add(remain_fund);

        if final_from_amount.is_some() && final_to_amount.is_some() {
            **ctx
                .accounts
                .giveaway_pool
                .to_account_info()
                .lamports
                .borrow_mut() = final_from_amount.unwrap();

            **ctx.accounts.payer.to_account_info().lamports.borrow_mut() = final_to_amount.unwrap();
        }

        let giveaway_id_str = hex::encode(args.giveaway_id.to_vec());

        let log_msg = format!(
            "{},{},{},{},{}",
            event_type::EventType::Refund as u32,
            giveaway_id_str,
            remain_fund,
            ctx.accounts.payer.key.to_string(),
            "0",
        );

        utils::log(log_msg);

        Ok(())
    }

    pub fn refund_token(
        ctx: Context<RefundTokenGiveawayAccount>,
        args: RefundPutGiveawayARGS,
    ) -> Result<()> {
        require!(
            ctx.accounts
                .payer
                .key()
                .as_ref()
                .eq(ctx.accounts.giveaway_pool.creator.key().as_ref()),
            GiveawayError::Forbidden
        );

        let (_pool, _bump) = Pubkey::find_program_address(
            &[
                &ctx.accounts.payer.key().to_bytes(),
                &ctx.accounts.token_mint.key().to_bytes(),
            ],
            ctx.program_id,
        );

        let remain_amount = ctx.accounts.giveaway_pool.total_amount;

        let transfer_instruction = ppl_token::instruction::transfer(
            ctx.accounts.token_program.key,
            &ctx.accounts.token_pool.key(),
            &ctx.accounts.to_account.key(),
            &ctx.accounts.token_pool.key(),
            &[&ctx.accounts.token_pool.key()],
            remain_amount,
        )?;
        put_anchor_lang::put_program::program::invoke_signed(
            &transfer_instruction,
            &[
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.token_pool.to_account_info(),
                ctx.accounts.to_account.to_account_info(),
            ],
            &[&[
                ctx.accounts.payer.key.as_ref(),
                ctx.accounts.token_mint.key().as_ref(),
                &[_bump],
            ]],
        )?;

        let giveaway_id_str = hex::encode(args.giveaway_id.to_vec());

        let log_msg = format!(
            "{},{},{},{},{}",
            event_type::EventType::Receive as u32,
            giveaway_id_str,
            remain_amount,
            ctx.accounts.payer.key.to_string(),
            ctx.accounts.token_mint.key().to_string(),
        );

        utils::log(log_msg);

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(args: CreateGiveawayARG)]
pub struct CreateNonPutGiveawayAccounts<'info> {
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
    #[account(mut)]
    payer: Signer<'info>,
    /// CHECK:
    #[account(mut, token::mint = token_mint)]
    from_account: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(init_if_needed, payer = payer, token::mint = token_mint, token::authority = token_pool, seeds = [&payer.key().to_bytes(), &token_mint.key().to_bytes()], bump)]
    token_pool: Account<'info, TokenAccount>,
    token_mint: Account<'info, Mint>,

    #[account(init_if_needed, payer = payer, space = 32 + 8 + 16 + usize::try_from(args.giveaway_count).unwrap() * 32, seeds = [&args.giveaway_id.clone()], bump)]
    giveaway_pool: Account<'info, GiveawayPool>,
}

#[derive(Accounts)]
#[instruction(args: CreateGiveawayARG)]
pub struct CreatePutGiveawayAccounts<'info> {
    system_program: Program<'info, System>,
    #[account(mut)]
    payer: Signer<'info>,
    #[account(init_if_needed, payer = payer, space = 32 + 8 + 16 + 4 + usize::try_from(args.giveaway_count).unwrap() * 32, seeds = [&args.giveaway_id.clone()], bump)]
    giveaway_pool: Account<'info, GiveawayPool>,
}

#[derive(Accounts)]
#[instruction(args: ReceivePutGiveawayARG)]
pub struct ReceivePutGiveawayAccount<'info> {
    system_program: Program<'info, System>,
    #[account(mut)]
    payer: Signer<'info>,
    #[account(mut, seeds = [&args.giveaway_id.clone()], bump)]
    giveaway_pool: Account<'info, GiveawayPool>,
}

// PDA 账户
#[account]
pub struct GiveawayPool {
    creator: Pubkey,
    receive_records: Vec<Pubkey>, // 领取记录，记录领取的钱包
    total_amount: u128,           // 红包总金额
}

// impl GiveawayPool {
//     pub const MAX_SIZE: usize = 32 + 4 + 16;
// }

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateGiveawayARG {
    giveaway_id: [u8; 20], // 红包ID
    giveaway_count: u32,
    amount: u128, // 红包总金额
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ReceivePutGiveawayARG {
    giveaway_id: [u8; 20],    // 红包ID
    wallet_address: [u8; 32], // 领取钱包地址
    amount: u128,
    timestamp: u64,
    signature: [u8; 65],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ReceiveNonPutGiveawayARG {
    giveaway_id: [u8; 20], // 红包ID
    amount: u128,
    timestamp: u64,
    signature: [u8; 65],
}

#[derive(Accounts)]
#[instruction(args: ReceiveNonPutGiveawayARG)]
pub struct ReceiveNonPutGiveawayAccount<'info> {
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    #[account(mut)]
    payer: Signer<'info>,
    #[account(mut, seeds = [&args.giveaway_id.clone()], bump)]
    giveaway_pool: Account<'info, GiveawayPool>,
    /// CHECK:
    #[account(mut, token::mint = token_mint, token::authority = token_pool, seeds = [&payer.key().to_bytes(), &token_mint.key().to_bytes()], bump)]
    token_pool: Account<'info, TokenAccount>,
    #[account(mut)]
    to_account: Account<'info, TokenAccount>,
    token_mint: Account<'info, Mint>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RefundPutGiveawayARGS {
    giveaway_id: [u8; 20], // 红包ID
}

#[derive(Accounts)]
#[instruction(args: RefundPutGiveawayARGS)]
pub struct RefundPutGiveawayAccount<'info> {
    system_program: Program<'info, System>,
    #[account(mut)]
    payer: Signer<'info>,
    #[account(mut, seeds = [&args.giveaway_id.clone()], bump)]
    giveaway_pool: Account<'info, GiveawayPool>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RefundTokenArgs {
    giveaway_id: [u8; 20], // 红包ID
}

#[derive(Accounts)]
#[instruction(args: RefundTokenArgs)]
pub struct RefundTokenGiveawayAccount<'info> {
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    #[account(mut)]
    payer: Signer<'info>,
    #[account(mut, seeds = [&args.giveaway_id.clone()], bump)]
    giveaway_pool: Account<'info, GiveawayPool>,
    /// CHECK:
    #[account(mut, token::mint = token_mint, token::authority = token_pool, seeds = [&payer.key().to_bytes(), &token_mint.key().to_bytes()], bump)]
    token_pool: Account<'info, TokenAccount>,
    #[account(mut)]
    to_account: Account<'info, TokenAccount>,
    token_mint: Account<'info, Mint>,
}
