/**
 * Clawnch Protocol - AI Agent-owned Memecoin Launchpad
 *
 * @description
 * Agents launch memecoins with real tokenomics:
 * - Fee collection on every trade
 * - Automatic buybacks (price support)
 * - Staking rewards for holders
 * - Creator revenue share
 * - Transparent on-chain treasury
 *
 * @license MIT
 * @author ClawdbotKV
 */

// Core token logic
export { ClawnchToken, TokenConfig, CLAWNCH_SEEDS, findProtocolPDA } from './token.js';

// Jupiter integration for buybacks
export {
  ClawnchJupiter,
  BuybackConfig,
} from './jupiter.js';

// Staking system
export {
  ClawnchStaking,
  StakeAccount,
  REWARD_RATES,
} from './staking.js';

// API server (when running Node)
if (typeof require !== 'undefined') {
  require('./api.js');
}

// Version
export const VERSION = '1.0.0';

/**
 * Fee distribution breakdown
 *
 * Every 2% fee is split as:
 * - 10% → Clawnch Protocol (revenue)
 * - 20% → Creator (incentive)
 * - 35% → Buyback (price support)
 * - 35% → Stakers (holder rewards)
 */
export const FEE_DISTRIBUTION = {
  PROTOCOL: 10,
  CREATOR: 20,
  BUYBACK: 35,
  STAKING: 35,
  TOTAL: 100,
} as const;

/**
 * Configuration constants
 */
export const CONFIG = {
  DEFAULT_FEE_BPS: 200, // 2%
  DEFAULT_DECIMALS: 9,
  MIN_STAKE_AMOUNT: 1_000_000, // 1 token
  DEFAULT_SLIPPAGE_BPS: 50, // 0.5%
} as const;
