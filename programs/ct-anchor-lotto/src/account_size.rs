pub fn get_configuration_size() -> usize {
    let discriminator = 8;
    let admin = 32;
    let buffer = 8;

    discriminator + admin + buffer
}

pub fn get_lottery_size() -> usize {
    let discriminator = 8; // Anchor discriminator

    let switchboard_feed = 32; // Pubkey
    let tickets_sold = 4; // u32
    let lamports_per_ticket = 8; // u64

    // Strings in your struct: start + end hex
    // Max hex length = 6 chars for your config (e.g., "fffff")
    // Anchor stores String as 4 bytes len + UTF-8 bytes
    let start_hex = 4 + 6;
    let end_hex = 4 + 6;

    let winner_settled = 1; // bool
    let refunds_settled = 4; // u32
    let open = 1; // bool
    let platform_fee_percentage = 2; // u16

    let buffer = 32; // safety buffer

    discriminator
        + switchboard_feed
        + tickets_sold
        + lamports_per_ticket
        + start_hex
        + end_hex
        + winner_settled
        + refunds_settled
        + open
        + platform_fee_percentage
        + buffer
}

pub fn get_transaction_bundle_size() -> usize {
    let discriminator = 8;
    let lottery = 32;
    let owner = 32;
    let tickets = 100 * 8;
    let refudned = 1;
    let buffer = 16;

    discriminator + lottery + owner + tickets + refudned + buffer
}
