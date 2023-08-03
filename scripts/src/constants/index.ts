import { BN } from '@coral-xyz/anchor'
import { SingleLevelAmmValidator } from '@jup-ag/core/dist/lib/ammValidator'
import { PublicKey } from '@solana/web3.js'

export { Clad } from '@/target/types/clad'

// export const JUPITER_PROGRAM_ID = 'JUP3c2Uh3WA4Ng34tw6kPd2G4C5BB21Xo36Je1s32Ph' // v3
export const JUPITER_PROGRAM_ID = 'JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB' // v4

export const TICK_ARRAY_SIZE = 88

export const MAX_SWAP_TICK_ARRAYS = 3

/**
 * The max tick index supported by Globalpools.
 */
export const MAX_TICK_INDEX = 443636

/**
 * The min tick index supported by Globalpools.
 */
export const MIN_TICK_INDEX = -443636

/**
 * The max sqrt-price supported by Globalpools.
 */
export const MAX_SQRT_PRICE = '79226673515401279992447579055'

/**
 * The min sqrt-price supported by Globalpools.
 */
export const MIN_SQRT_PRICE = '4295048016'

/**
 * The denominator which the protocol fee rate is divided on.
 */
export const PROTOCOL_FEE_RATE_MUL_VALUE = new BN(10_000)

/**
 * The denominator which the fee rate is divided on.
 */
export const FEE_RATE_MUL_VALUE = new BN(1_000_000)

export const ZERO_BN = new BN(0)

export const tokenMintSOL = new PublicKey(
  'So11111111111111111111111111111111111111112'
)

export const tokenMintUSDC = new PublicKey(
  'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'
)

export const tokenMintBONK = new PublicKey(
  'DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263'
)

export const tokenMintFIDA = new PublicKey(
  'EchesyfXePKdLtoiZSL8pBe8Myagyy8ZRqsACNCFGnvp'
)

export const tokenMintHNT = new PublicKey(
  'hntyVP6YFm1Hg25TN9WGLqM12b8TQmcknKrdu1oxWux'
)

export const tokenMintIOT = new PublicKey(
  'iotEVVZLEywoTn1QdwNPddxPWszn3zFhEot3MfL9fns'
)

export const ORCA = new PublicKey('orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE')

export const WBTC = new PublicKey(
  '9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E'
)

export const WETH = new PublicKey(
  '7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs'
)

export const MSOL = new PublicKey('mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So')

export const MNDE = new PublicKey('MNDEFzGvMt87ueuHvVU9VcTqsAP5b3fTGPsHuuPA5ey')

export const SAMO = new PublicKey(
  '7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU'
)

export const pythOracleSOL = new PublicKey(
  'H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG'
)

export const pythOracleUSDC = new PublicKey(
  'Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD'
)

export const testJupiterAmmsToExclude: SingleLevelAmmValidator = {
  Aldrin: true,
  Crema: true,
  Cropper: true,
  Cykura: true,
  DeltaFi: true,
  GooseFX: true,
  Invariant: true,
  Lifinity: true,
  'Lifinity V2': true,
  Marinade: true,
  Mercurial: true,
  Meteora: true,
  Orca: false,
  'Orca (Whirlpools)': false,
  Raydium: true,
  'Raydium CLMM': true,
  Saber: true,
  Serum: true,
  Step: true,
  Penguin: true,
  Saros: true,
  Stepn: true,
  Sencha: true,
  'Saber (Decimals)': true,
  Dradex: true,
  Balansol: true,
  Openbook: true,
  Oasis: true,
  BonkSwap: true,
  Phoenix: true,
  Symmetry: true,
  Unknown: true,
}
