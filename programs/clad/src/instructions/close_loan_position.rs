use {
    crate::{
        errors::ErrorCode,
        manager::loan_manager,
        state::*,
        util::{burn_and_close_user_position_token, verify_position_authority},
    },
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{self, Mint, Token, TokenAccount},
    },
};

#[derive(Accounts)]
pub struct CloseLoanPosition<'info> {
    #[account(mut)]
    pub position_authority: Signer<'info>,

    pub globalpool: Box<Account<'info, Globalpool>>,

    /// CHECK: safe, for receiving rent only
    #[account(mut)]
    pub receiver: UncheckedAccount<'info>,

    #[account(
        mut,
        close = receiver,
        seeds = [
            b"trade_position".as_ref(),
            position_mint.key().as_ref()
        ],
        bump,
	)]
    pub position: Account<'info, TradePosition>,

    #[account(mut, address = position.position_mint)]
    pub position_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = position_token_account.amount == 1,
        constraint = position_token_account.mint == position.position_mint
    )]
    pub position_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut, has_one = globalpool)]
    pub tick_array_lower: AccountLoader<'info, TickArray>,

    #[account(mut, has_one = globalpool)]
    pub tick_array_upper: AccountLoader<'info, TickArray>,

    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
}

pub fn close_loan_position(ctx: Context<CloseLoanPosition>) -> Result<()> {
    verify_position_authority(
        &ctx.accounts.position_token_account,
        &ctx.accounts.position_authority,
    )?;

    if !TradePosition::is_position_empty(&ctx.accounts.position) {
        return Err(ErrorCode::CloseTradePositionNotEmpty.into());
    }

    let borrow_a = ctx.accounts.position.is_borrow_a(&ctx.accounts.globalpool);

    // Add numbers back to `liquidity_available` of each borrowed ticks
    // Decrease numbers from `liquidity_borrowed_a` and `liquidity_borrowed_b`
    let tick_lower_index = position.tick_lower_index;
    let tick_upper_index = position.tick_upper_index;

    let tick_array_lower = tick_array_lower.load()?;
    let tick_lower = tick_array_lower.get_tick(tick_lower_index, globalpool.tick_spacing)?;

    let tick_array_upper = tick_array_upper.load()?;
    let tick_upper = tick_array_upper.get_tick(tick_upper_index, globalpool.tick_spacing)?;

    //
    // Burn loan position token
    //

    burn_and_close_user_position_token(
        &ctx.accounts.position_authority,
        &ctx.accounts.receiver,
        &ctx.accounts.position_mint,
        &ctx.accounts.position_token_account,
        &ctx.accounts.token_program,
    )
}
