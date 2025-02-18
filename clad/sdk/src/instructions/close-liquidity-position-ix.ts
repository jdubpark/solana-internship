import { Program } from '@coral-xyz/anchor'
import { Instruction } from '@orca-so/common-sdk'
import { TOKEN_PROGRAM_ID } from '@solana/spl-token'
import { PublicKey } from '@solana/web3.js'

import { Clad } from '@/target/types/clad'

/**
 * Parameters to close a position in a Whirlpool.
 *
 * @category Instruction Types
 * @param receiver - PublicKey for the wallet that will receive the rented lamports.
 * @param position - PublicKey for the position.
 * @param positionMint - PublicKey for the mint token for the Position token.
 * @param positionTokenAccount - The associated token address for the position token in the owners wallet.
 * @param positionAuthority - Authority that owns the position token.
 */
export type CloseLiquidityPositionAccounts = {
  receiver: PublicKey
  position: PublicKey
  positionMint: PublicKey
  positionTokenAccount: PublicKey
  positionAuthority: PublicKey
}

/**
 * Close a position in a Whirlpool. Burns the position token in the owner's wallet.
 *
 * @category Instructions
 * @param context - Context object containing services required to generate the instruction
 * @param params - ClosePositionParams object
 * @returns - Instruction to perform the action.
 */
export function closeLiquidityPositionIx(
  program: Program<Clad>,
  accouts: CloseLiquidityPositionAccounts
): Instruction {
  const ix = program.instruction.closeLiquidityPosition({
    accounts: {
      ...accouts,
      tokenProgram: TOKEN_PROGRAM_ID,
    },
  })

  return {
    instructions: [ix],
    cleanupInstructions: [],
    signers: [],
  }
}
