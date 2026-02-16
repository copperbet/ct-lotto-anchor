use crate::account_size::*;
use crate::account_struct::*;
use crate::lotto_enum::LottoError;
use anchor_lang::prelude::*;

/* -------------------------------------------------
   CREATE CONFIGURATION PDA
--------------------------------------------------*/
#[derive(Accounts)]
pub struct CreateConfigurationPDA<'info> {
    #[account(
        init,
        payer = admin,
        seeds = [b"configuration"],
        bump,
        space = get_configuration_size()
    )]
    pub configuration: Account<'info, Configuration>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/* -------------------------------------------------
   CREATE LOTTERY PDA
--------------------------------------------------*/
#[derive(Accounts)]
#[instruction(lottery_seed: String)]
pub struct CreateLotteryPDA<'info> {
    pub configuration: Account<'info, Configuration>,

    #[account(
        init,
        payer = admin,
        seeds = [
            b"lottery",
            lottery_seed.as_bytes()
        ],
        bump,
        space = get_lottery_size()
    )]
    pub lottery: Account<'info, Lottery>,

    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK
    pub switchboard_feed_btc_block_decimal: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

/* -------------------------------------------------
   CREATE TRANSACTION BUNDLE
   PDA = ["bundle", lottery_seed, tx_sig_hash]
--------------------------------------------------*/
#[derive(Accounts)]
#[instruction(lottery_seed: String, tx_sig_hash: [u8; 32])]
pub struct CreateTransactionBundle<'info> {
    pub configuration: Account<'info, Configuration>,

    #[account(mut)]
    pub lottery: Account<'info, Lottery>,

    #[account(
        init,
        payer = admin,
        seeds = [
            b"bundle",
            lottery_seed.as_bytes(),
            &tx_sig_hash
        ],
        bump,
        space = get_transaction_bundle_size()
    )]
    pub bundle: Account<'info, TransactionBundle>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/* -------------------------------------------------
   REWARD USING A TRANSACTION BUNDLE
--------------------------------------------------*/
#[derive(Accounts)]
pub struct RewardTransactionBundle<'info> {
    pub configuration: Account<'info, Configuration>,

    #[account(mut)]
    pub lottery: Account<'info, Lottery>,

    #[account(mut)]
    pub bundle: Account<'info, TransactionBundle>,

    /// CHECK
    pub switchboard_feed_btc_block_decimal: AccountInfo<'info>,

    /// CHECK
    #[account(mut)]
    pub owner: AccountInfo<'info>,

    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/* -------------------------------------------------
   REFUND USING A TRANSACTION BUNDLE
--------------------------------------------------*/
#[derive(Accounts)]
pub struct RefundTransactionBundle<'info> {
    pub configuration: Account<'info, Configuration>,

    #[account(mut)]
    pub lottery: Account<'info, Lottery>,

    #[account(mut)]
    pub bundle: Account<'info, TransactionBundle>,

    /// CHECK: refund target
    #[account(mut)]
    pub owner: AccountInfo<'info>,

    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/* -------------------------------------------------
   CLOSE LOTTERY
--------------------------------------------------*/
#[derive(Accounts)]
pub struct CloseLottery<'info> {
    pub configuration: Account<'info, Configuration>,

    #[account(mut)]
    pub lottery: Account<'info, Lottery>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/* -------------------------------------------------
   CLOSE TRANSACTION BUNDLE
   PDA = ["bundle", lottery_seed, tx_sig_hash]
--------------------------------------------------*/
#[derive(Accounts)]
pub struct CloseTransactionBundle<'info> {
    pub configuration: Account<'info, Configuration>,

    #[account(
        mut,
        close = admin   // rent goes to admin
    )]
    pub bundle: Account<'info, TransactionBundle>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/* -------------------------------------------------
   CLOSE LOTTERY PDA
--------------------------------------------------*/
#[derive(Accounts)]
pub struct CloseLotteryPDA<'info> {
    pub configuration: Account<'info, Configuration>,

    #[account(mut, close = admin)]
    pub lottery: Account<'info, Lottery>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/* -------------------------------------------------
   CLOSE CONFIGURATION PDA
--------------------------------------------------*/
#[derive(Accounts)]
pub struct CloseConfigurationPDA<'info> {
    #[account(mut, close = admin)]
    pub configuration: Account<'info, Configuration>,

    // we're transfering the rent to the admin, to allow only admin to close the account we ask for signature.
    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}
