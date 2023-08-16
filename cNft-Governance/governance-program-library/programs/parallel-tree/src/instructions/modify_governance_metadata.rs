use anchor_lang::prelude::*;
use crate::error::ParallelTreeError;
use crate::utils::get_asset_id::get_parallel_asset_id;
use anchor_lang::solana_program::keccak;
use spl_account_compression::{ Noop, program::SplAccountCompression };
use spl_account_compression::cpi::accounts::VerifyLeaf;
use crate::state::*;

/// `ModifyGovernanceMetadata`: An instruction to modify existing governance metadata in a parallel Merkle tree.
/// This instruction facilitates updating the content of an existing leaf within the Merkle tree, ensuring the
/// provided root aligns and the authority is valid. The update requires verifying the previous content of the leaf
/// before replacing it with the new metadata. If the tree is private, modifications can only be made by the tree
/// creator or its delegate.
///
#[derive(Accounts)]
pub struct ModifyGovernanceMetadata<'info> {
    #[account(seeds = [parallel_tree.key().as_ref()], bump)]
    pub parallel_tree_authority: Account<'info, TreeConfig>,

    #[account(mut)]
    /// CHECK: This account is checked in the instruction
    pub parallel_tree: UncheckedAccount<'info>,
    /// CHECK: This account is checked in the instruction
    pub leaf_owner: AccountInfo<'info>,
    /// CHECK: This account is checked in the instruction
    pub leaf_delegate: AccountInfo<'info>,
    pub tree_delegate: Signer<'info>,

    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub system_program: Program<'info, System>,
}

pub fn modify_governance_metadata<'info>(
    ctx: Context<'_, '_, '_, 'info, ModifyGovernanceMetadata<'info>>,
    root: [u8; 32],
    nonce: u64,
    index: u32, // still don't understand why need both index and nonce
    data_hash: [u8; 32],
    message: GovernanceMetadata
) -> Result<()> {
    let parallel_tree = &ctx.accounts.parallel_tree.to_account_info();
    let leaf_owner = &ctx.accounts.leaf_owner.to_account_info();
    let leaf_delegate = &ctx.accounts.leaf_delegate.to_account_info();
    let compression_program = &ctx.accounts.compression_program.to_account_info();
    let authority = &mut ctx.accounts.parallel_tree_authority;
    let tree_creator = authority.tree_creator;
    let tree_delegate = authority.tree_delegate;
    let incoming_tree_delegate = ctx.accounts.tree_delegate.key();
    let proofs = &ctx.remaining_accounts.to_vec();

    if !authority.is_public {
        require!(
            incoming_tree_delegate == tree_creator || incoming_tree_delegate == tree_delegate,
            ParallelTreeError::TreeAuthorityIncorrect
        );
    }

    let asset_id = get_parallel_asset_id(&parallel_tree.key(), &message.compressed_nft);
    let previous_leaf = LeafSchema::new_v0(
        asset_id,
        *leaf_owner.key,
        *leaf_delegate.key,
        nonce,
        data_hash
    );
    let cpi_ctx = CpiContext::new(compression_program.clone(), VerifyLeaf {
        merkle_tree: parallel_tree.clone(),
    }).with_remaining_accounts(proofs.clone());

    spl_account_compression::cpi::verify_leaf(
        cpi_ctx,
        root.clone(),
        previous_leaf.to_node(),
        index
    )?;

    let data_hash = keccak::hashv(&[message.try_to_vec()?.as_slice()]).to_bytes();

    let new_leaf = LeafSchema::new_v0(
        asset_id,
        *leaf_owner.key,
        *leaf_delegate.key,
        nonce,
        data_hash
    );

    let seed = parallel_tree.key();
    let seeds = &[seed.as_ref(), &[*ctx.bumps.get("parallel_tree_authority").unwrap()]];
    let authority_pda_signer = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(
        compression_program.clone(),
        spl_account_compression::cpi::accounts::Modify {
            authority: ctx.accounts.parallel_tree_authority.to_account_info(),
            merkle_tree: parallel_tree.clone(),
            noop: ctx.accounts.log_wrapper.to_account_info(),
        },
        authority_pda_signer
    ).with_remaining_accounts(proofs.clone());
    spl_account_compression::cpi::replace_leaf(
        cpi_ctx,
        root,
        previous_leaf.to_node(),
        new_leaf.to_node(),
        index
    )?;
    Ok(())
}
