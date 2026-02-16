use anchor_lang::prelude::*;

#[account]
pub struct Configuration {
    /// Global admin of the entire lottery program.
    pub admin: Pubkey,
}

#[account]
pub struct Lottery {
    /// Switchboard feed used to read the BTC block height (in decimal).
    pub switchboard_feed_btc_block_decimal: Pubkey,

    /// Total number of tickets sold for this lottery.
    pub tickets_sold: u32,

    /// Lamports charged per ticket.
    pub lamports_per_ticket: u64,

    /// Starting hex (e.g. "0", "00", "000").
    pub ticket_code_start_hex: String,

    /// Ending hex (e.g. "F", "FF", "FFF").
    pub ticket_code_end_hex: String,

    /// Whether winner payout is completed.
    pub winner_settled: bool,

    /// Number of refunds completed.
    pub refunds_settled: u32,

    /// Whether lottery is open for new purchases.
    pub open: bool,

    /// Platform fee in percentage (0–100).
    pub platform_fee_percentage: u16,
}

#[account]
pub struct TransactionBundle {
    /// Lottery this bundle belongs to.
    pub lottery_pda: Pubkey,

    /// The owner of all the tickets in this bundle.
    pub owner: Pubkey,

    /// Each ticket is stored as hex bytes (max 8 chars).  
    /// Example: "1A2B3C" -> [ '1','A','2','B','3','C',0,0 ]
    pub tickets: [[u8; 8]; 100],

    /// To know whether the refund as been issues
    pub refunded: bool,
}
