use put_anchor_lang::prelude::*;

#[error_code]
pub enum ReelPayError {
    #[msg("Forbidden")]
    Forbidden,
    #[msg("Args Error")]
    ArgsError,
}
