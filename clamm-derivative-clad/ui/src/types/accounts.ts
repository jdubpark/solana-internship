import { BN } from '@coral-xyz/anchor'
import { Account, Mint } from '@solana/spl-token'
import { PublicKey } from '@solana/web3.js'

export enum AccountName {
  LiquidityPosition = 'LiquidityPosition',
  TradePosition = 'TradePosition',
  TickArray = 'TickArray',
  Globalpool = 'Globalpool',
}

export type BasicSupportedTypes = Account | Mint

export type GlobalpoolData = {
  bump: number
  tickSpacing: number
  tickSpacingSeed: number
  feeRate: number
  feeRateSeed: number
  protocolFeeRate: number
  liquidityAvailable: BN
  liquidityBorrowed: BN
  sqrtPrice: BN
  tickCurrentIndex: number
  protocolFeeOwedA: BN
  protocolFeeOwedB: BN
  tokenMintA: PublicKey
  tokenVaultA: PublicKey
  feeGrowthGlobalA: BN
  tokenMintB: PublicKey
  tokenVaultB: PublicKey
  feeGrowthGlobalB: BN
  inceptionTime: BN
  feeAuthority: PublicKey
}

export type LiquidityPositionData = {
  globalpool: PublicKey
  positionMint: PublicKey
  liquidity: BN
  tickLowerIndex: number
  tickUpperIndex: number
  feeGrowthCheckpointA: BN
  feeOwedA: BN
  feeGrowthCheckpointB: BN
  feeOwedB: BN
}

export type TradePositionData = {
  globalpool: PublicKey
  positionMint: PublicKey
  tickLowerIndex: number
  tickUpperIndex: number
  tickOpenIndex: number,
  liquidityBorrowed: BN // u128
  loanTokenAvailable: BN // u64
  loanTokenSwapped: BN // u64
  tradeTokenAmount: BN // u64
  collateralAmount: BN // u64
  tokenMintLoan: PublicKey
  tokenMintCollateral: PublicKey
  openTime: BN // u64
  duration: BN // u64
  interestRate: number // u32
}

export type TickData = {
  initialized: boolean
  liquidityNet: BN
  liquidityGross: BN
  liquidityBorrowed: BN
  feeGrowthOutsideA: BN
  feeGrowthOutsideB: BN
}

export type TickArrayData = {
  globalpool: PublicKey
  startTickIndex: number
  ticks: TickData[]
}

export type CladSupportedTypes =
  | GlobalpoolData
  | LiquidityPositionData
  | TradePositionData
  | TickArrayData
  | BasicSupportedTypes

/**
 * Extended Mint type to host token info.
 */
export type TokenInfo = Mint & { mint: PublicKey }

/**
 * Extended (token) Account type to host account info for a Token.
 */
export type TokenAccountInfo = Account

/**
 * A wrapper class of a TickArray on a Globalpool
 */
export type TickArray = {
  address: PublicKey
  data: TickArrayData | null
}
