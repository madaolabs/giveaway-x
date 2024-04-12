use put_anchor_lang::{prelude::*, put_program::program_pack::Pack};

declare_id!("2b3hfMSNDeFAfVTuoFe9pQaJJzM1gV1jGjjv5rWACh7U");

#[program]
pub mod wallet {
    use super::*;

    #[error_code]
    pub enum BalanceError {
        #[msg("parameter must be non-empty")]
        ParameterMustNonEmpty,
    }

    pub fn get_token_balance(ctx: Context<TokenMutAccount>) -> Result<Vec<TokenBalance>> {
        let mut result = Vec::new();
        require!(
            ctx.remaining_accounts.len() > 0,
            BalanceError::ParameterMustNonEmpty
        );
        let wallet_account = ctx.remaining_accounts.get(0).unwrap();
        let address = wallet_account.key.to_string();
        result.push(TokenBalance {
            address,
            balance: wallet_account.lamports(),
        });

        ctx.remaining_accounts.iter().skip(1).for_each(|account| {
            let is_empty = account.try_data_is_empty();
            match is_empty {
                Ok(is_empty) => {
                    if !is_empty {
                        let token =
                            <put_anchor_ppl::token::ppl_token::state::Account as Pack>::unpack(
                                *account.data.borrow(),
                            );
                        match token {
                            Ok(token) => {
                                let balance = token.amount;
                                let address = account.key.to_string();
                                result.push(TokenBalance { address, balance });
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        });
        Ok(result)
    }

    #[derive(Accounts)]
    pub struct TokenMutAccount {}
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct TokenBalance {
    address: String,
    balance: u128,
}
