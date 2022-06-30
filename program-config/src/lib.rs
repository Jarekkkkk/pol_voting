use bytemuck::{from_bytes, from_bytes_mut, Pod};
use solana_program::{account_info::AccountInfo, program_error::ProgramError};
use std::cell::{Ref, RefMut};
pub trait Loadable: Pod {
    fn load<'a>(acc: &'a AccountInfo) -> Result<Ref<'a, Self>, ProgramError> {
        Ok(Ref::map(acc.try_borrow_data()?, |data| from_bytes(data)))
    }

    fn load_mut<'a>(acc: &'a AccountInfo) -> Result<RefMut<'a, Self>, ProgramError> {
        Ok(RefMut::map(acc.try_borrow_mut_data()?, |data| {
            from_bytes_mut(data)
        }))
    }
}
