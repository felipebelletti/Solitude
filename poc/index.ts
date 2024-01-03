import { PublicKey } from "@solana/web3.js";
import { write, writeFile, writeFileSync } from "fs";

const web3 = require("@solana/web3.js");

/*
Test and real data:
Token: Etpju1XBPwjSgQ5tcdPEKsLZwmwmGNonUbacUBjMf2gF
BaseMint: Etpju1XBPwjSgQ5tcdPEKsLZwmwmGNonUbacUBjMf2gF
QuoteMint: So11111111111111111111111111111111111111112

{"id":"8CTgrTG9qe4iK4iKGyidbUii7MYS4uzX2gmE2bXDa6WV","baseMint":"Etpju1XBPwjSgQ5tcdPEKsLZwmwmGNonUbacUBjMf2gF","quoteMint":"So11111111111111111111111111111111111111112","lpMint":"DitUDr63iJTEVpAN8oX1mJyLGH4QwqFKoUuqp5xb4xLA","baseDecimals":6,"quoteDecimals":9,"lpDecimals":6,"version":4,"programId":"675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8","authority":"5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1","openOrders":"C8nu3jebcFscEUe4jHBSKMUBYeab8jmDo54CTwYFc9gE","targetOrders":"5VeCmU9nj8zbFMTyVvRKGnKrR1HuzNLAVqZguyFipvQJ","baseVault":"GbbqncbV7cxphfuXsYMYrsvd7HTX9qJx23Jmw5sEjz47","quoteVault":"jAQD81qSGJUGczhh2Vdnvxj68SyyezJ8giGG4zCCDFi","withdrawQueue":"11111111111111111111111111111111","lpVault":"11111111111111111111111111111111","marketVersion":4,"marketProgramId":"srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX","marketId":"2ozU23yywPR4uhTKQwUUMp936rPtLdxc4Fsdy6LibPWH","marketAuthority":"38MoDrijmybQFeZikMGegZ2bTBzASbG34peztbkjcEPx","marketBaseVault":"FGoMpWDmxRXh4SiMFkUichRow6Z58215AwhVNFZ4uLLh","marketQuoteVault":"HaSmnKmukZeBKyD6dhwfUDL5oTJAEza1aSH1BjYSrSkt","marketBids":"NgtgsLU19YDHJm8QD79DDC9sVBTA28wjUrjNc8jQEmL","marketAsks":"J4waFJhwAJ7P21ECHquxZDQCesFiBy4t3u2qMNVdrEPp","marketEventQueue":"5k1nfotLp4W1neE2wkZgsLrQFJUcoUoSfouRx3W1RfGp"},

________________________________________________________________________________________

Token: AuqvqC8NhrGMUJaJwUqoYkAnxyQqKZA6tF52ZxmnFTmS
BaseMint: AuqvqC8NhrGMUJaJwUqoYkAnxyQqKZA6tF52ZxmnFTmS
QuoteMint: So11111111111111111111111111111111111111112
LPMint: 2jxc3ntvEd7nbegViQH92C91iQUTfJMmprDnn5oEHMah
ProgramId: 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8
Authority: 5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1
OpenOrders: HBFRsHMkaQCJrmXh6CnSwFEAdhNJPAWknHAobcL2t64Q
TargetOrders: 3jCTzX4pmoyr9Wxe1CiHaM8it3ZHPS3WNdtwAwwD8ZBi

{"id":"Ho9rEHJoaB3T9j6Anv2AH75ftUsLrJWvn4ffEN3GiYBn","baseMint":"AuqvqC8NhrGMUJaJwUqoYkAnxyQqKZA6tF52ZxmnFTmS","quoteMint":"So11111111111111111111111111111111111111112","lpMint":"2jxc3ntvEd7nbegViQH92C91iQUTfJMmprDnn5oEHMah","baseDecimals":3,"quoteDecimals":9,"lpDecimals":3,"version":4,"programId":"675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8","authority":"5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1","openOrders":"HBFRsHMkaQCJrmXh6CnSwFEAdhNJPAWknHAobcL2t64Q","targetOrders":"3jCTzX4pmoyr9Wxe1CiHaM8it3ZHPS3WNdtwAwwD8ZBi","baseVault":"C72p29iFxFKDgzYrYexRsXHZm4uyJfHgTXaap1wYUdKa","quoteVault":"4xs2C45PntyLP5KaSaLaxjMoNGaseq2D5q2R6YrzDxw6","withdrawQueue":"11111111111111111111111111111111","lpVault":"11111111111111111111111111111111","marketVersion":4,"marketProgramId":"srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX","marketId":"6HXQpNkcy9KKkxWi8rhCfxfdLZQuDAj7HjUqRY244mC3","marketAuthority":"C1cPxTExbSsjC7xhhgQgD8JndqzgsHSBVdGrzCWo5783","marketBaseVault":"4UqwHi7PYXqjDQzej9Qu4vfowm4zycwgX2sQwu2tGfR6","marketQuoteVault":"BbXehiDafme9ZYrxpHiPWedtTqBwm2ZgMJGEBvoXmfSu","marketBids":"7NPj9YNeTH6JxSYn5HRpbjKGnsC95dAx1GehVoTDr8Nz","marketAsks":"GSSqqMuSY8s6cxkVaqvKNWDYT1tapfMns8mZ6BKBGwmc","marketEventQueue":"BTH36w3BkjaCDiAKzwoRYwtPnEJ4jENHTZTkAhy9EBfk"}
*/

// Define the connection to the Solana cluster
const connection = new web3.Connection(
  // "https://ancient-chaotic-aura.solana-mainnet.quiknode.pro/316c14d6ae6c6f4a358f0a44b3e34244e6e34783"
  "https://tame-ancient-mountain.solana-mainnet.quiknode.pro/6a9a95bf7bbb108aea620e7ee4c1fd5e1b67cc62"
);
const openbookProgram = new PublicKey(
  "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX"
);
const raydium_liquidity_pool_v4_program = new PublicKey("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

async function analyze() {
  // Analyze Raydium Market
  // let marketAddress = new PublicKey('Ho9rEHJoaB3T9j6Anv2AH75ftUsLrJWvn4ffEN3GiYBn')
  // let marketAddress = new PublicKey('6HXQpNkcy9KKkxWi8rhCfxfdLZQuDAj7HjUqRY244mC3'); // test
  let real_market_data: any = {"id":"Ho9rEHJoaB3T9j6Anv2AH75ftUsLrJWvn4ffEN3GiYBn","baseMint":"AuqvqC8NhrGMUJaJwUqoYkAnxyQqKZA6tF52ZxmnFTmS","quoteMint":"So11111111111111111111111111111111111111112","lpMint":"2jxc3ntvEd7nbegViQH92C91iQUTfJMmprDnn5oEHMah","baseDecimals":3,"quoteDecimals":9,"lpDecimals":3,"version":4,"programId":"675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8","authority":"5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1","openOrders":"HBFRsHMkaQCJrmXh6CnSwFEAdhNJPAWknHAobcL2t64Q","targetOrders":"3jCTzX4pmoyr9Wxe1CiHaM8it3ZHPS3WNdtwAwwD8ZBi","baseVault":"C72p29iFxFKDgzYrYexRsXHZm4uyJfHgTXaap1wYUdKa","quoteVault":"4xs2C45PntyLP5KaSaLaxjMoNGaseq2D5q2R6YrzDxw6","withdrawQueue":"11111111111111111111111111111111","lpVault":"11111111111111111111111111111111","marketVersion":4,"marketProgramId":"srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX","marketId":"6HXQpNkcy9KKkxWi8rhCfxfdLZQuDAj7HjUqRY244mC3","marketAuthority":"C1cPxTExbSsjC7xhhgQgD8JndqzgsHSBVdGrzCWo5783","marketBaseVault":"4UqwHi7PYXqjDQzej9Qu4vfowm4zycwgX2sQwu2tGfR6","marketQuoteVault":"BbXehiDafme9ZYrxpHiPWedtTqBwm2ZgMJGEBvoXmfSu","marketBids":"7NPj9YNeTH6JxSYn5HRpbjKGnsC95dAx1GehVoTDr8Nz","marketAsks":"GSSqqMuSY8s6cxkVaqvKNWDYT1tapfMns8mZ6BKBGwmc","marketEventQueue":"BTH36w3BkjaCDiAKzwoRYwtPnEJ4jENHTZTkAhy9EBfk"};
  // let market = await connection.getAccountInfo(marketAddress);

  // Analyze Openbook Market (extracted form the previous RaydiuMarket->marketId, notice they are using the same real market data object)
  // let real_market_data: any = {
  //   id: "Ho9rEHJoaB3T9j6Anv2AH75ftUsLrJWvn4ffEN3GiYBn",
  //   baseMint: "AuqvqC8NhrGMUJaJwUqoYkAnxyQqKZA6tF52ZxmnFTmS",
  //   quoteMint: "So11111111111111111111111111111111111111112",
  //   lpMint: "2jxc3ntvEd7nbegViQH92C91iQUTfJMmprDnn5oEHMah",
  //   baseDecimals: 3,
  //   quoteDecimals: 9,
  //   lpDecimals: 3,
  //   version: 4,
  //   programId: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
  //   authority: "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1",
  //   openOrders: "HBFRsHMkaQCJrmXh6CnSwFEAdhNJPAWknHAobcL2t64Q",
  //   targetOrders: "3jCTzX4pmoyr9Wxe1CiHaM8it3ZHPS3WNdtwAwwD8ZBi",
  //   baseVault: "C72p29iFxFKDgzYrYexRsXHZm4uyJfHgTXaap1wYUdKa",
  //   quoteVault: "4xs2C45PntyLP5KaSaLaxjMoNGaseq2D5q2R6YrzDxw6",
  //   withdrawQueue: "11111111111111111111111111111111",
  //   lpVault: "11111111111111111111111111111111",
  //   marketVersion: 4,
  //   marketProgramId: "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX",
  //   marketId: "6HXQpNkcy9KKkxWi8rhCfxfdLZQuDAj7HjUqRY244mC3",
  //   marketAuthority: "C1cPxTExbSsjC7xhhgQgD8JndqzgsHSBVdGrzCWo5783",
  //   marketBaseVault: "4UqwHi7PYXqjDQzej9Qu4vfowm4zycwgX2sQwu2tGfR6",
  //   marketQuoteVault: "BbXehiDafme9ZYrxpHiPWedtTqBwm2ZgMJGEBvoXmfSu",
  //   marketBids: "7NPj9YNeTH6JxSYn5HRpbjKGnsC95dAx1GehVoTDr8Nz",
  //   marketAsks: "GSSqqMuSY8s6cxkVaqvKNWDYT1tapfMns8mZ6BKBGwmc",
  //   marketEventQueue: "BTH36w3BkjaCDiAKzwoRYwtPnEJ4jENHTZTkAhy9EBfk",
  // };
  // let marketAddress = new PublicKey(real_market_data.marketId);
  // let market = await connection.getAccountInfo(marketAddress);

  // console.log(market);

  // analyze_market(market, real_market_data);

  // test
  for (const address of Object.values(real_market_data)) {
    if(typeof address !== "string") continue;
    let market = await connection.getAccountInfo(new PublicKey(address));
    analyze_market(market, real_market_data, address);
  }
}

const analyze_market = (market: any, real_market_data: any, scanned_addr: any) => {
  delete real_market_data["withdrawQueue"]; // wildcard like (should be removed)
  delete real_market_data["lpVault"]; // wildcard like (should be removed)
  const real_market_data_tokens = Object.values(real_market_data).map((val) =>
    typeof val === "string" ? val.toLowerCase() : val
  );
  const mentioned_data = [];

  let x = 0;
  let y = 32;

  if(!market || !market.data) return;

  // Find offsets
  while (y <= market.data.length) {
    let sliced = new PublicKey(market.data.subarray(x, y)).toString();
    writeFileSync("/tmp/shittery.data", `${sliced} | scanned: ${scanned_addr} | offset: [${x}:${y}]\n`, { flag: "a" });
    if (real_market_data_tokens.includes(sliced.toLowerCase())) {
      const key_name = Object.keys(real_market_data).find(
        (key: string) => real_market_data[key] === sliced
      );
      mentioned_data.push(key_name);
      console.log(`Offset ${x} to ${y} is ${sliced} (${key_name})`);
    }
    x++;
    y++;
  }

  for (let key in real_market_data) {
    if (!mentioned_data.includes(key)) {
      console.log(`Key ${key} not found`);
    }
  }
};

const findMarkets = async () => {
  // extracting markets
  const openbook_datasize = 388; // Data size of market accounts
  const raydium_pool_datasize = 752;
  const targetTokenAddress = new PublicKey(
    // "AuqvqC8NhrGMUJaJwUqoYkAnxyQqKZA6tF52ZxmnFTmS"
    "HCwwJh3Gx99KqmE6oH2J4t1wLc8dWWfK1Jy3eytg7Rc9"
  );

  // find openbook

  let openbook_baseMintAccounts = await connection.getProgramAccounts(openbookProgram, {
    filters: [
      { dataSize: openbook_datasize },
      {
        memcmp: {
          offset: 53,
          bytes: targetTokenAddress.toBase58(),
        },
      },
    ],
  });
  console.log(openbook_baseMintAccounts, targetTokenAddress.toBase58());
  await new Promise((resolve) => setTimeout(resolve, 3000));
  let openbook_quoteMintAccounts = await connection.getProgramAccounts(openbookProgram, {
    filters: [
      { dataSize: openbook_datasize },
      {
        memcmp: {
          offset: 85,
          bytes: targetTokenAddress.toBase58(),
        },
      },
    ],
  });
  let openbook_accounts = [...openbook_baseMintAccounts, ...openbook_quoteMintAccounts];

  if (!openbook_accounts.length) {
    console.log("No accounts found");
    return;
  }
  if (openbook_accounts.length > 1) {
    console.log("Multiple accounts found");
    return;
  }

  console.log(openbook_accounts);
  const market_account = openbook_accounts[0].account;
  console.log(market_account);
  const openbook_market = await enumerate_market_account(
    market_account
  );

  // find raydium pool
  await new Promise((resolve) => setTimeout(resolve, 3000));
  let baseMintAccounts = await connection.getProgramAccounts(raydium_liquidity_pool_v4_program, {
    filters: [
      { dataSize: raydium_pool_datasize },
      {
        memcmp: {
          offset: 400,
          bytes: targetTokenAddress.toBase58(),
        },
      },
    ],
  });
  await new Promise((resolve) => setTimeout(resolve, 3000));
  let quoteMintAccounts = await connection.getProgramAccounts(raydium_liquidity_pool_v4_program, {
    filters: [
      { dataSize: raydium_pool_datasize },
      {
        memcmp: {
          offset: 432,
          bytes: targetTokenAddress.toBase58(),
        },
      },
    ],
  });
  let accounts = [...baseMintAccounts, ...quoteMintAccounts];

  if (!accounts) {
    console.log("No accounts found");
    return;
  }
  if (accounts.length > 1) {
    console.log("Multiple pool accounts found");
    return;
  }

  const raydium_pool_account = accounts[0].account;
  
  const raydium_pool = enumerate_raydium_pool(raydium_pool_account);
  console.log(openbook_market);
  console.log(raydium_pool);
};

// openbook
const enumerate_market_account = async (market_account: any) => {
  const marketId = new PublicKey(market_account.data.subarray(13, 45));
  const baseMint = new PublicKey(market_account.data.subarray(53, 85));
  const quoteMint = new PublicKey(market_account.data.subarray(85, 117));
  const marketBaseVault = new PublicKey(market_account.data.subarray(117, 149));
  const marketQuoteVault = new PublicKey(
    market_account.data.subarray(165, 197)
  );
  const marketEventQueue = new PublicKey(
    market_account.data.subarray(253, 285)
  );
  const marketBids = new PublicKey(market_account.data.subarray(285, 317));
  const marketAsks = new PublicKey(market_account.data.subarray(317, 349));

  return {
    marketId,
    baseMint,
    quoteMint,
    marketBaseVault,
    marketQuoteVault,
    marketEventQueue,
    marketBids,
    marketAsks,
  };
};

const enumerate_raydium_pool = (pool_account: any) => {
    /*
    - Offset 336 to 368 is C72p29iFxFKDgzYrYexRsXHZm4uyJfHgTXaap1wYUdKa (baseVault)
- Offset 368 to 400 is 4xs2C45PntyLP5KaSaLaxjMoNGaseq2D5q2R6YrzDxw6 (quoteVault)
- Offset 400 to 432 is AuqvqC8NhrGMUJaJwUqoYkAnxyQqKZA6tF52ZxmnFTmS (baseMint)
- Offset 432 to 464 is So11111111111111111111111111111111111111112 (quoteMint)
- Offset 464 to 496 is 2jxc3ntvEd7nbegViQH92C91iQUTfJMmprDnn5oEHMah (lpMint)
- Offset 496 to 528 is HBFRsHMkaQCJrmXh6CnSwFEAdhNJPAWknHAobcL2t64Q (openOrders)
- Offset 528 to 560 is 6HXQpNkcy9KKkxWi8rhCfxfdLZQuDAj7HjUqRY244mC3 (marketId)
- Offset 560 to 592 is srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX (marketProgramId)
- Offset 592 to 624 is 3jCTzX4pmoyr9Wxe1CiHaM8it3ZHPS3WNdtwAwwD8ZBi (targetOrders)

    */

    const baseMint = new PublicKey(pool_account.data.subarray(400, 432));
    const quoteMint = new PublicKey(pool_account.data.subarray(432, 464));
    const lpMint = new PublicKey(pool_account.data.subarray(464, 496));
    const baseVault = new PublicKey(pool_account.data.subarray(336, 368));
    const quoteVault = new PublicKey(pool_account.data.subarray(368, 400));
    const openOrders = new PublicKey(pool_account.data.subarray(496, 528));
    const marketId = new PublicKey(pool_account.data.subarray(528, 560));
    const marketProgramId = new PublicKey(pool_account.data.subarray(560, 592));
    const targetOrders = new PublicKey(pool_account.data.subarray(592, 624));

    return {
        baseMint,
        quoteMint,
        lpMint,
        baseVault,
        quoteVault,
        openOrders,
        marketId,
        marketProgramId,
        targetOrders
    }
};

(async () => {
  await analyze();
  // await findMarkets();
})();