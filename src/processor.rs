use base58::FromBase58;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};

/// Define the type of state stored in accounts
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CounterTracker {
    pub count: u32,
}

/// Instruction processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let global_counter = next_account_info(accounts_iter)?;
    let user = next_account_info(accounts_iter)?;
    let user_counter = next_account_info(accounts_iter)?;

    assert!(user.is_signer);

    let auth_ctr_prog_id = {
        let auth_ctr_prog = "Af4DodBScfdZ8Qyq7qf6sNFisJQopQ93adTawWvNxvFK";
        let auth_ctr_prog = auth_ctr_prog.from_base58().unwrap();
        let mut auth_ctr_prog_arr = [0_u8; 32];
        auth_ctr_prog_arr.copy_from_slice(auth_ctr_prog.as_slice());
        Pubkey::new_from_array(auth_ctr_prog_arr)
    };

    let user_counter_seeds = &[user.key.as_ref(), global_counter.key.as_ref()];
    let (user_counter_pda, bump) = Pubkey::find_program_address(user_counter_seeds, program_id);
    assert_eq!(&user_counter_pda, user_counter.key);

    match instruction_data.split_first() {
        Some((&0, &[])) => {
            let system_program_info = next_account_info(accounts_iter)?;
            invoke_signed(
                &system_instruction::create_account(
                    user.key,
                    user_counter.key,
                    100000000,
                    36,
                    &auth_ctr_prog_id,
                ),
                &[
                    user.clone(),
                    user_counter.clone(),
                    system_program_info.clone(),
                ],
                &[&[user.key.as_ref(), global_counter.key.as_ref(), &[bump]]],
            )?
        }
        Some((&1, &[])) => {
            invoke_signed(
                &Instruction {
                    program_id: auth_ctr_prog_id,
                    accounts: vec![
                        AccountMeta::new(*user_counter.key, false),
                        AccountMeta::new(*user_counter.key, true),
                    ],
                    data: vec![],
                },
                &[user_counter.clone()],
                &[&[user.key.as_ref(), global_counter.key.as_ref(), &[bump]]],
            )?;
            let mut tracker = CounterTracker::try_from_slice(&global_counter.data.borrow())?;
            tracker.count += 1;
            tracker.serialize(&mut &mut global_counter.data.borrow_mut()[..])?;
        }
        _ => return Err(ProgramError::InvalidInstructionData),
    }
    Ok(())
}
