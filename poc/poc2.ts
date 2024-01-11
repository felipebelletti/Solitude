import {
  Liquidity,
  Market,
  LIQUIDITY_STATE_LAYOUT_V4,
  MarketV2,
  getMultipleAccountsInfo,
  MARKET_STATE_LAYOUT_V3,
  MARKET_STATE_LAYOUT_V2,
} from "@raydium-io/raydium-sdk";
import { Connection, PublicKey } from "@solana/web3.js";
import moment from "moment";

/*
export const MAINNET_PROGRAM_ID: ProgramId = {
  SERUM_MARKET: new PublicKey('9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin'),
  OPENBOOK_MARKET: new PublicKey('srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX'),

  UTIL1216: new PublicKey('CLaimxFqjHzgTJtAGHU47NPhg6qrc5sCnpC4tBLyABQS'),

  FarmV3: new PublicKey('EhhTKczWMGQt46ynNeRX1WfeagwwJd7ufHvCDjRxjo5Q'),
  FarmV5: new PublicKey('9KEPoZmtHUrBbhWN1v1KWLMkkvwY6WLtAVUCPRtRjP4z'),
  FarmV6: new PublicKey('FarmqiPv5eAj3j1GMdMCMUGXqPUvmquZtMy86QH6rzhG'),

  AmmV4: new PublicKey('675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8'),
  AmmStable: new PublicKey('5quBtoiQqxF9Jv6KYKctB59NT3gtJD2Y65kdnB1Uev3h'),

  CLMM: new PublicKey('CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK'),

  Router: new PublicKey('routeUGWgWzqBWFcrCfv8tritsqukccJPu3q5GPP3xS'),
}
*/

( () => {
    const programId = new PublicKey("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");
    const marketId = new PublicKey("3sJVHtBTjpHmTgArbtTz5cDr6umJZUTjx8yZfyoGAhZm");

    const raydium_pool_addr = Liquidity.getAssociatedId({ programId, marketId })
    const base_vault = Liquidity.getAssociatedBaseVault({ programId, marketId })

    const authority = Liquidity.getAssociatedAuthority({ programId })

    console.log(raydium_pool_addr);
    console.log(base_vault);
    console.log(authority);
})();