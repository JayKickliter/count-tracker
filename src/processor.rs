use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Define the type of state stored in accounts
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AccountString {
    pub memo: [u8; 6],
}

/// Instruction processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;

    if account.owner != program_id {
        msg!("this program does not own account {:?}", account);
        return Err(ProgramError::IncorrectProgramId);
    }

    let acct_memo = AccountString {
        memo: {
            let mut acct_str = [0; 6];
            acct_str.copy_from_slice(&instruction_data[..6]);
            acct_str
        },
    };

    acct_memo.serialize(&mut &mut account.data.borrow_mut()[..])?;

    Ok(())
}
