use anchor_lang::prelude::*;

#[error_code]
pub enum LottoError {
    /* ------------------------------ */
    /*  GENERAL ACCESS ERRORS         */
    /* ------------------------------ */
    #[msg("Only admin can perform this action.")]
    AdminOnlyAction,

    /* ------------------------------ */
    /*  LOTTERY ERRORS                */
    /* ------------------------------ */
    #[msg("Lottery is closed.")]
    LotteryClosed,

    #[msg("Cannot close PDA because the required conditions are not met.")]
    PDACloseConditionNotMet,

    #[msg("Invalid Transaction request.")]
    DuplicateRequest,

    #[msg("Key mismatch.")]
    KeyMismatch,

    #[msg("The lottery has not tickets to issue refund.")]
    NoTicketSoldToRefund,

    /* ------------------------------ */
    /*  TRANSACTION BUNDLE ERRORS     */
    /* ------------------------------ */
    #[msg("Transaction bundle already contains the maximum limit of 100 tickets.")]
    BundleFull,

    #[msg("Hex exceeds maximum allowed length of 8 characters.")]
    HexTooLong,

    #[msg("Ticket not found inside this transaction bundle.")]
    TicketNotInBundle,

    #[msg("The bundle owner does not match the provided owner account.")]
    WinnerMismatch,

    /* ------------------------------ */
    /*  RANGE / VALIDATION ERRORS     */
    /* ------------------------------ */
    #[msg("Requested ticket code is outside the valid lottery range.")]
    TicketCodeOutOfRange,

    /* ------------------------------ */
    /*  SWITCHBOARD FEED ERRORS       */
    /* ------------------------------ */
    #[msg("Switchboard feed account mismatch.")]
    SwitchboardFeedMismatch,

    #[msg("Unable to parse Switchboard feed account data.")]
    FeedParseError,

    #[msg("Switchboard feed contains no value yet.")]
    FeedNoValue,

    #[msg("Switchboard feed value mismatch.")]
    FeedValueMismatch,

    /* ------------------------------ */
    /*  LAMPORT TRANSFER ERRORS       */
    /* ------------------------------ */
    #[msg("Source account does not have enough lamports.")]
    InsufficientLamports,
}
