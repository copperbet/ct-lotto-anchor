use anchor_lang::prelude::*;
use switchboard_on_demand::PullFeedAccountData;

use crate::lotto_enum::LottoError;

/* -------------------------------------------------
   READ SWITCHBOARD FEED (BTC block height decimal)
--------------------------------------------------*/
pub fn read_feed_value(feed_account: &AccountInfo) -> Result<[u8; 8]> {
    let data = feed_account.data.borrow();

    let feed = PullFeedAccountData::parse(data).map_err(|_| LottoError::FeedParseError)?;

    // When feed is not updated
    require!(feed.last_update_timestamp != 0, LottoError::FeedNoValue);

    // Convert decimal-with-18-places → integer
    let int_value = feed.result.value / 10i128.pow(18);

    // Safety: ensure value fits into u64 (8 bytes)
    require!(
        int_value >= 0 && int_value <= u64::MAX as i128,
        LottoError::HexTooLong
    );

    let value = int_value as u64;

    // Convert u64 → 8-byte big-endian representation
    let bytes = value.to_be_bytes();

    Ok(bytes) // exactly [u8; 8]
}

/* -------------------------------------------------
   SAFE LAMPORT TRANSFER
   - Checks balance
   - No rent-exemption issues for PDA paying out
   - Direct lamport movement
--------------------------------------------------*/
pub fn transfer_lamports<'info>(
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    lamports: u64,
) -> Result<()> {
    require!(
        **from.lamports.borrow() >= lamports,
        LottoError::InsufficientLamports
    );

    // subtract from source
    **from.try_borrow_mut_lamports()? -= lamports;

    // add to destination
    **to.try_borrow_mut_lamports()? += lamports;

    Ok(())
}
