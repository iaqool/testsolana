use anchor_lang::prelude::*;

declare_id!("EYfHSdmUTkcXEt2rsUUdW16C9taGPR8sXjMsQqV4F5pZ");

#[program]
pub mod mytoken {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
