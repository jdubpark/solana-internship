import { TransactionBuilder, Wallet } from '@orca-so/common-sdk'
import { Connection, Signer, TransactionInstruction } from '@solana/web3.js'

export function createTransactionChained(
  conenction: Connection,
  wallet: Wallet,
  instructions: TransactionInstruction | TransactionInstruction[],
  signers: Signer[]
): TransactionBuilder

export function createTransactionChained(
  conenction: Connection,
  wallet: Wallet,
  instructions: TransactionInstruction | TransactionInstruction[],
  cleanupInstructions: TransactionInstruction[]
): TransactionBuilder

export function createTransactionChained(
  conenction: Connection,
  wallet: Wallet,
  instructions: TransactionInstruction | TransactionInstruction[],
  cleanupInstructions: TransactionInstruction[],
  signers: Signer[]
): TransactionBuilder

export function createTransactionChained(
  conenction: Connection,
  wallet: Wallet,
  instructions: TransactionInstruction | TransactionInstruction[],
  cleanupInstructionsOrSigners?: TransactionInstruction[] | Signer[],
  signers?: Signer[]
) {
  let _cleanupInstructions: TransactionInstruction[] = []
  let _signers: Signer[] = []

  if (Array.isArray(cleanupInstructionsOrSigners)) {
    if ((cleanupInstructionsOrSigners as Signer[])[0].publicKey !== undefined) {
      _signers = cleanupInstructionsOrSigners as Signer[]
    } else {
      _cleanupInstructions =
        cleanupInstructionsOrSigners as TransactionInstruction[]
      _signers = signers ?? []
    }
  }

  return new TransactionBuilder(conenction, wallet).addInstruction({
    instructions: Array.isArray(instructions) ? instructions : [instructions],
    cleanupInstructions: _cleanupInstructions,
    signers: _signers,
  })
}
