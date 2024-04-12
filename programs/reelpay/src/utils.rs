use put_anchor_lang::prelude::*;

pub fn log(content: String) {
    msg!("ReelPay {}", hex::encode(content));
}
