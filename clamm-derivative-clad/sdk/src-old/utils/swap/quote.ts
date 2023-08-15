import { BN } from '@coral-xyz/anchor'

import {
  MAX_SQRT_PRICE,
  MAX_SWAP_TICK_ARRAYS,
  MIN_SQRT_PRICE,
  ZERO_BN,
} from '@/constants'
import { computeSwap } from '@/utils/swap/compute-swap'
import { SwapQuote, SwapQuoteParam } from '@/utils/swap/types'
import { TickArraySequence } from '@/utils/tick-array-sequence'

/**
 * Figure out the quote parameters needed to successfully complete this trade on chain
 * @param param
 * @returns
 * @exceptions
 */
export function simulateSwap(params: SwapQuoteParam): SwapQuote {
  const {
    aToB,
    globalpoolData,
    tickArrays,
    tokenAmount,
    sqrtPriceLimit,
    otherAmountThreshold,
    amountSpecifiedIsInput,
  } = params

  if (
    sqrtPriceLimit.gt(new BN(MAX_SQRT_PRICE)) ||
    sqrtPriceLimit.lt(new BN(MIN_SQRT_PRICE))
  ) {
    throw new Error('Provided SqrtPriceLimit is out of bounds.')
  }

  const sqrtPriceBN = new BN(globalpoolData.sqrtPrice.toString())

  if (
    (aToB && sqrtPriceLimit.gt(sqrtPriceBN)) ||
    (!aToB && sqrtPriceLimit.lt(sqrtPriceBN))
  ) {
    throw new Error(
      'Provided SqrtPriceLimit is in the opposite direction of the trade.'
    )
  }

  if (tokenAmount.eq(ZERO_BN)) {
    throw new Error('Provided tokenAmount is ZERO_BN.')
  }

  const tickSequence = new TickArraySequence(
    tickArrays,
    globalpoolData.tickSpacing,
    aToB
  )

  // Ensure 1st search-index resides on the 1st array in the sequence to match smart contract expectation.
  if (!tickSequence.isValidTickArray0(globalpoolData.tickCurrentIndex)) {
    throw new Error(
      'TickArray at index 0 does not contain the Whirlpool current tick index.'
    )
  }

  const swapResults = computeSwap(
    globalpoolData,
    tickSequence,
    tokenAmount,
    sqrtPriceLimit,
    amountSpecifiedIsInput,
    aToB
  )

  if (amountSpecifiedIsInput) {
    if (
      (aToB && otherAmountThreshold.gt(swapResults.amountB)) ||
      (!aToB && otherAmountThreshold.gt(swapResults.amountA))
    ) {
      throw new Error(
        'Quoted amount for the other token is below the otherAmountThreshold.'
      )
    }
  } else {
    if (
      (aToB && otherAmountThreshold.lt(swapResults.amountA)) ||
      (!aToB && otherAmountThreshold.lt(swapResults.amountB))
    ) {
      throw new Error(
        'Quoted amount for the other token is above the otherAmountThreshold.'
      )
    }
  }

  const { estimatedAmountIn, estimatedAmountOut } = remapAndAdjustTokens(
    swapResults.amountA,
    swapResults.amountB,
    aToB
  )

  const numOfTickCrossings = tickSequence.getNumOfTouchedArrays()
  if (numOfTickCrossings > MAX_SWAP_TICK_ARRAYS) {
    throw new Error(
      `Input amount causes the quote to traverse more than the allowable amount of tick-arrays ${numOfTickCrossings}`
    )
  }

  const touchedArrays = tickSequence.getTouchedArrays(MAX_SWAP_TICK_ARRAYS)

  return {
    estimatedAmountIn,
    estimatedAmountOut,
    estimatedEndTickIndex: swapResults.nextTickIndex,
    estimatedEndSqrtPrice: swapResults.nextSqrtPrice,
    estimatedFeeAmount: swapResults.totalFeeAmount,
    amount: tokenAmount,
    amountSpecifiedIsInput,
    aToB,
    otherAmountThreshold,
    sqrtPriceLimit,
    tickArray0: touchedArrays[0],
    tickArray1: touchedArrays[1],
    tickArray2: touchedArrays[2],
  }
}

function remapAndAdjustTokens(amountA: BN, amountB: BN, aToB: boolean) {
  const estimatedAmountIn = aToB ? amountA : amountB
  const estimatedAmountOut = aToB ? amountB : amountA
  return {
    estimatedAmountIn,
    estimatedAmountOut,
  }
}
