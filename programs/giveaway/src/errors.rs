use put_anchor_lang::prelude::*;

#[error_code]
pub enum GiveawayError {
    #[msg("Forbidden")]
    Forbidden,
    #[msg("Args Error")]
    ArgsError,
    #[msg("Overtime")]
    Overtime,
    #[msg("Exceed")]
    ExceedError,
}
