import express from "express";
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

const app = express();
app.use(express.json());

// const rpc_url = "http://127.0.0.1:8899/"
const rpc_url = "http://api.mainnet-beta.solana.com/"
const web3Conn = new Connection(rpc_url, {
  commitment: "processed",
});

async function craftPoolKey(base_mint: string, quote_mint: string, target_token: string) {
  let isTargetTokenTheBaseMint = base_mint.toLowerCase() === target_token.toLowerCase();

  const apiQuery = await fetch(`https://openserum.io/api/serum/token/${target_token}`);
  const apiQueryJson = await apiQuery.json() as any;
  let selectedJsonInfo = apiQueryJson.findLast(
    (el: any) => el.baseMint.toLowerCase() === target_token.toLowerCase()
    || el.quoteMint.toLowerCase() === target_token.toLowerCase()
  );

  const { value: baseTokenInfo } = await web3Conn.getTokenSupply(new PublicKey(selectedJsonInfo.baseMint));
  const { value: quoteTokenInfo } = await web3Conn.getTokenSupply(new PublicKey(selectedJsonInfo.quoteMint));

  //console.log(selectedJsonInfo)

  const programId = Liquidity.getProgramId(4);
  const marketId = new PublicKey(selectedJsonInfo.id);
  const marketProgramId = new PublicKey(
    "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX"
  ); // its really a constante value

  const craftedPoolKey /*: LiquidityPoolKeys*/ = {
    id: Liquidity.getAssociatedId({ programId, marketId }),
    baseMint: new PublicKey(selectedJsonInfo.baseMint),
    quoteMint: new PublicKey(selectedJsonInfo.quoteMint),
    lpMint: Liquidity.getAssociatedLpMint({ marketId, programId }),

    baseDecimals: baseTokenInfo.decimals,
    quoteDecimals: quoteTokenInfo.decimals,
    lpDecimals: isTargetTokenTheBaseMint ? quoteTokenInfo.decimals : baseTokenInfo.decimals,
    version: 4,

    marketEventQueue: new PublicKey(selectedJsonInfo.eventQueue),

    // here is the problem
    marketBaseVault: Liquidity.getAssociatedBaseVault({
      programId: marketProgramId,
      marketId,
    }),
    marketQuoteVault: Liquidity.getAssociatedQuoteVault({
      programId: marketProgramId,
      marketId,
    }),
    authority: new PublicKey(
      Liquidity.getAssociatedAuthority({ programId }).publicKey
    ),

    programId: programId,
    openOrders: Liquidity.getAssociatedOpenOrders({ programId, marketId }),
    targetOrders: Liquidity.getAssociatedTargetOrders({ programId, marketId }),
    baseVault: Liquidity.getAssociatedBaseVault({ programId, marketId }),
    quoteVault: Liquidity.getAssociatedQuoteVault({ programId, marketId }),
    withdrawQueue: Liquidity.getAssociatedWithdrawQueue({
      programId,
      marketId,
    }),
    lpVault: Liquidity.getAssociatedLpVault({ programId, marketId }),

    marketVersion: 4,
    marketProgramId: marketProgramId,
    marketId: marketId,
    marketAuthority: new PublicKey(
      Market.getAssociatedAuthority({
        programId: marketProgramId,
        marketId,
      }).publicKey
    ),

    marketAsks: new PublicKey(selectedJsonInfo.asks),
    marketBids: new PublicKey(selectedJsonInfo.bids),
  };

  return craftedPoolKey;
}

( async () => {
    let a = await web3Conn.getAccountInfo(new PublicKey("7zVfZZ8sRU3sMxMfXXjUPmc2tURxYD5tG5MJeBeSN8Jy")).then((a) => a!);
    console.log(MARKET_STATE_LAYOUT_V3.decode(a.data));

    // let a = await craftPoolKey("Etpju1XBPwjSgQ5tcdPEKsLZwmwmGNonUbacUBjMf2gF", "So11111111111111111111111111111111111111112", "Etpju1XBPwjSgQ5tcdPEKsLZwmwmGNonUbacUBjMf2gF");
    // console.log(a);
})();