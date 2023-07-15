import * as anchor from '@coral-xyz/anchor'
import { MathUtil, Percentage, TransactionBuilder } from '@orca-so/common-sdk'
import { PriceMath, TickUtil } from '@orca-so/whirlpools-sdk'
import { getMint } from '@solana/spl-token'
import { Connection, Keypair, PublicKey } from '@solana/web3.js'
import * as fs from 'fs'

import { Clad } from '@/target/types/clad'
import { tokenMintSOL, tokenMintUSDC } from './constants'

const fsp = fs.promises

export function getPDA(
  seeds: (Buffer | Uint8Array)[],
  programId: PublicKey
): PublicKey {
  return PublicKey.findProgramAddressSync(seeds, programId)[0]
}

export async function getConstantParams() {
  const provider = anchor.AnchorProvider.local('http://127.0.0.1:8899', {
    commitment: 'confirmed',
    preflightCommitment: 'confirmed',
  })

  // TODO: use dummy keypair as signer & wallet (NodeWallet)
  //       need to modify all provider.wallet and transactionbuilder instances

  const { connection, wallet } = provider

  const program = anchor.workspace.Clad as anchor.Program<Clad>
  const programId = program.programId

  const tickSpacing = 64
  const feeRate = 3000 // per 1_000_000 (3000 => 0.3%)

  const tokenMintAKey = tokenMintSOL
  const tokenMintBKey = tokenMintUSDC

  const tokenMintA = await getMint(connection, tokenMintAKey)
  const tokenMintB = await getMint(connection, tokenMintBKey)

  const initTickIndex = -39424 // 19.4054 B/A (USDC/SOL)
  const initPrice = PriceMath.tickIndexToPrice(
    initTickIndex,
    tokenMintA.decimals,
    tokenMintB.decimals
  )
  const initSqrtPrice = PriceMath.tickIndexToSqrtPriceX64(initTickIndex)

  const [cladKey] = PublicKey.findProgramAddressSync(
    [Buffer.from('clad')],
    programId
  )

  return {
    provider,
    program,
    programId,
    connection,
    wallet,
    keypair,
    feeRate,
    tickSpacing,
    tokenMintA,
    tokenMintB,
    cladKey,
    initTickIndex,
    initPrice,
    initSqrtPrice,
  }
}

export async function getPostPoolInitParams() {
  const cParams = await getConstantParams()

  const globalpoolSeeds = [
    Buffer.from('globalpool'),
    cParams.tokenMintA.address.toBuffer(),
    cParams.tokenMintB.address.toBuffer(),
    new anchor.BN(cParams.feeRate).toArrayLike(Buffer, 'le', 2),
    new anchor.BN(cParams.tickSpacing).toArrayLike(Buffer, 'le', 2),
  ]

  const [globalpoolKey] = PublicKey.findProgramAddressSync(
    globalpoolSeeds,
    cParams.programId
  )

  return {
    ...cParams,
    globalpoolKey,
  }
}
