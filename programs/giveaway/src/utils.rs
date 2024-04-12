use put_anchor_lang::prelude::*;

pub fn log(content: String) {
    msg!("Giveaway {}", hex::encode(content));
}
