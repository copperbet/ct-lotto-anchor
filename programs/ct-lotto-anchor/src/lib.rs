pub mod account_instruction;
pub mod account_size;
pub mod account_struct;
pub mod lotto_enum;
pub mod lotto_util;
use crate::account_instruction::*;
use crate::lotto_enum::*;
use crate::lotto_util::*;
use anchor_lang::prelude::*;
use solana_security_txt::security_txt;

declare_id!("CuNkHZPFXuz1FPLjLMsfpa3Zn41GmzK6vnJJV7EUQJ8z");

security_txt! {
    name: "lotto.copperbet.com",
    project_url: "https://lotto.copperbet.com",
    contacts: "email:admin@coppertech.in",
    policy: "",
    preferred_languages: "en",
    source_code: "https://github.com/copperbet/ct-lotto-anchor",
    auditors: "Nishanth D"
}

#[program]
pub mod ct_anchor_lotto {
    use super::*;

    /* -------------------------------------------------
       CREATE CONFIGURATION
    --------------------------------------------------*/
    pub fn create_configuration_pda(ctx: Context<CreateConfigurationPDA>) -> Result<()> {
        ctx.accounts.configuration.admin = ctx.accounts.admin.key();

        msg!(
            "CONFIGURATION_PDA_CREATED: {}",
            ctx.accounts.configuration.key()
        );

        Ok(())
    }

    /* -------------------------------------------------
       CREATE LOTTERY
    --------------------------------------------------*/
    pub fn create_lottery_pda(
        ctx: Context<CreateLotteryPDA>,
        _lottery_seed: String,
        lamports_per_ticket: u64,
        start_hex: String,
        end_hex: String,
        fee_percent: u16,
    ) -> Result<()> {
        require_eq!(
            ctx.accounts.admin.key(),
            ctx.accounts.configuration.admin,
            LottoError::AdminOnlyAction
        );

        ctx.accounts.lottery.switchboard_feed_btc_block_decimal =
            ctx.accounts.switchboard_feed_btc_block_decimal.key();

        ctx.accounts.lottery.tickets_sold = 0;
        ctx.accounts.lottery.refunds_settled = 0;
        ctx.accounts.lottery.winner_settled = false;

        ctx.accounts.lottery.open = true;

        ctx.accounts.lottery.lamports_per_ticket = lamports_per_ticket;
        ctx.accounts.lottery.ticket_code_start_hex = start_hex;
        ctx.accounts.lottery.ticket_code_end_hex = end_hex;

        ctx.accounts.lottery.platform_fee_percentage = fee_percent;

        msg!("LOTTERY_PDA_CREATED: {}", ctx.accounts.lottery.key());

        Ok(())
    }

    /* -------------------------------------------------
       CREATE TRANSACTION BUNDLE (with purchased numbers)
    --------------------------------------------------*/
    pub fn create_transaction_bundle(
        ctx: Context<CreateTransactionBundle>,
        _lottery_seed: String,
        _tx_sig_hash: [u8; 32],
        owner: Pubkey,
        purchased_numbers: Vec<[u8; 8]>,
    ) -> Result<()> {
        require_eq!(
            ctx.accounts.admin.key(),
            ctx.accounts.configuration.admin,
            LottoError::AdminOnlyAction
        );

        let bundle = &mut ctx.accounts.bundle;
        let lottery = &mut ctx.accounts.lottery;

        require!(purchased_numbers.len() <= 100, LottoError::BundleFull);

        // store owner + lottery link
        bundle.owner = owner;
        bundle.lottery_pda = lottery.key();

        // Store raw bytes
        for (i, ticket_bytes) in purchased_numbers.iter().enumerate() {
            bundle.tickets[i] = *ticket_bytes;
        }

        // update total sold tickets
        lottery.tickets_sold += purchased_numbers.len() as u32;

        msg!("TRANSACTION_BUNDLE_CREATED: {}", ctx.accounts.bundle.key());

        Ok(())
    }

    /* -------------------------------------------------
       REWARD WINNER USING BUNDLE
    --------------------------------------------------*/
    pub fn reward_transaction_bundle(
        ctx: Context<RewardTransactionBundle>,
        winning_bytes: [u8; 8],
    ) -> Result<()> {
        require_eq!(
            ctx.accounts.admin.key(),
            ctx.accounts.configuration.admin,
            LottoError::AdminOnlyAction
        );

        let lottery = &mut ctx.accounts.lottery;
        let bundle = &ctx.accounts.bundle;

        require!(!lottery.winner_settled, LottoError::DuplicateRequest);
        require!(lottery.refunds_settled == 0, LottoError::DuplicateRequest);

        // Verify correct Switchboard feed used
        require_keys_eq!(
            ctx.accounts.switchboard_feed_btc_block_decimal.key(),
            lottery.switchboard_feed_btc_block_decimal,
            LottoError::SwitchboardFeedMismatch
        );

        // read switchboard value
        let sb_winning_bytes = read_feed_value(&ctx.accounts.switchboard_feed_btc_block_decimal)
            .map_err(|_| LottoError::FeedParseError)?;

        msg!("winning_bytes = {:?}", winning_bytes);
        msg!("sb_winning_bytes = {:?}", sb_winning_bytes);

        // client value must equal switchboard feed value
        require!(
            winning_bytes == sb_winning_bytes,
            LottoError::FeedValueMismatch
        );

        // Verify this bundle belongs to this lottery and owner
        require_keys_eq!(bundle.lottery_pda, lottery.key(), LottoError::KeyMismatch);
        require_keys_eq!(
            bundle.owner,
            ctx.accounts.owner.key(),
            LottoError::WinnerMismatch
        );

        // Check if the winning ticket exists inside the bundle
        let found = bundle.tickets.iter().any(|t| *t == winning_bytes);

        require!(found, LottoError::TicketNotInBundle);

        // 4. Calculate payout
        let total_lamports = lottery.lamports_per_ticket * (lottery.tickets_sold as u64);
        let platform_fee = total_lamports * (lottery.platform_fee_percentage as u64) / 100;
        let payout_amount = total_lamports - platform_fee;

        // Transfer to winner
        transfer_lamports(
            &lottery.to_account_info(),
            &ctx.accounts.owner.to_account_info(),
            payout_amount,
        )?;

        lottery.winner_settled = true;

        msg!("TRANSACTION_BUNDLE_REWARDED: {}", ctx.accounts.bundle.key());

        Ok(())
    }

    /* -------------------------------------------------
       REFUND TICKETS USING BUNDLE
    --------------------------------------------------*/
    pub fn refund_transaction_bundle(
        ctx: Context<RefundTransactionBundle>,
        deduct_fee: bool,
    ) -> Result<()> {
        require_eq!(
            ctx.accounts.admin.key(),
            ctx.accounts.configuration.admin,
            LottoError::AdminOnlyAction
        );

        let lottery = &mut ctx.accounts.lottery;
        let bundle = &mut ctx.accounts.bundle;

        require!(lottery.tickets_sold > 0, LottoError::NoTicketSoldToRefund);

        require!(!bundle.refunded, LottoError::DuplicateRequest);

        require!(!lottery.winner_settled, LottoError::DuplicateRequest);

        require_keys_eq!(bundle.lottery_pda, lottery.key(), LottoError::KeyMismatch);

        let total_lamports = lottery.lamports_per_ticket * (lottery.tickets_sold as u64);

        let platform_fee = if deduct_fee {
            total_lamports * (lottery.platform_fee_percentage as u64) / 100
        } else {
            0
        };

        let refund_per_user = (total_lamports - platform_fee) / (lottery.tickets_sold as u64);

        transfer_lamports(
            &lottery.to_account_info(),
            &ctx.accounts.owner.to_account_info(),
            refund_per_user,
        )?;

        lottery.refunds_settled += 1;

        bundle.refunded = true;

        msg!("TRANSACTION_BUNDLE_REFUNDED: {}", bundle.key());

        Ok(())
    }

    /* -------------------------------------------------
       CLOSE LOTTERY
    --------------------------------------------------*/
    pub fn close_lottery(ctx: Context<CloseLottery>) -> Result<()> {
        require_eq!(
            ctx.accounts.admin.key(),
            ctx.accounts.configuration.admin,
            LottoError::AdminOnlyAction
        );

        ctx.accounts.lottery.open = false;

        msg!("LOTTERY_CLOSED: {}", ctx.accounts.lottery.key());

        Ok(())
    }

    /* -------------------------------------------------
       CLOSE TRANSACTION BUNDLE
    --------------------------------------------------*/
    pub fn close_transaction_bundle(ctx: Context<CloseTransactionBundle>) -> Result<()> {
        require_eq!(
            ctx.accounts.admin.key(),
            ctx.accounts.configuration.admin,
            LottoError::AdminOnlyAction
        );

        // Anchor automatically closes the bundle and returns rent
        msg!("TRANSACTION_BUNDLE_CLOSED: {}", ctx.accounts.bundle.key());

        Ok(())
    }

    /* -------------------------------------------------
       CLOSE LOTTERY PDA
    --------------------------------------------------*/
    pub fn close_lottery_pda(ctx: Context<CloseLotteryPDA>) -> Result<()> {
        // allow only admin to invoke this command
        require_eq!(
            ctx.accounts.admin.key(),
            ctx.accounts.configuration.admin.key(),
            LottoError::AdminOnlyAction
        );

        msg!("LOTTERY_PDA_CLOSED: {}", ctx.accounts.lottery.key());

        Ok(())
    }

    /* -------------------------------------------------
       CLOSE CONFIGURATION BUNDLE
    --------------------------------------------------*/
    pub fn close_configuration_pda(ctx: Context<CloseConfigurationPDA>) -> Result<()> {
        // allow only admin to invoke this command
        require_eq!(
            ctx.accounts.admin.key(),
            ctx.accounts.configuration.admin.key(),
            LottoError::AdminOnlyAction
        );

        msg!(
            "Attempting to close Configuration PDA: {}",
            ctx.accounts.configuration.key()
        );

        msg!(
            "CONFIGURATION_PDA_CLOSED: {}",
            ctx.accounts.configuration.key()
        );

        Ok(())
    }
}
