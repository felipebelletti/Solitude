Results from https://api.raydium.io/v2/sdk/liquidity/mainnet.json and https://openserum.io/api/serum/token/<token_address> differs on market id.
- https://api.raydium.io/v2/sdk/liquidity/mainnet.json id points to a raydium market id (aka the pool e.g bequiet-SOL) (owner is 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8)
- https://openserum.io/api/serum/token/<token_address> id points to a openbook market id (owner is srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX)

common sizes
- every raydium pool data structure has a dataSize (space) of 752
- found a openbook market data structure with 388 as dataSize

raydium pool data offsets
- Offset 336 to 368 is C72p29iFxFKDgzYrYexRsXHZm4uyJfHgTXaap1wYUdKa (baseVault)
- Offset 368 to 400 is 4xs2C45PntyLP5KaSaLaxjMoNGaseq2D5q2R6YrzDxw6 (quoteVault)
- Offset 400 to 432 is AuqvqC8NhrGMUJaJwUqoYkAnxyQqKZA6tF52ZxmnFTmS (baseMint)
- Offset 432 to 464 is So11111111111111111111111111111111111111112 (quoteMint)
- Offset 464 to 496 is 2jxc3ntvEd7nbegViQH92C91iQUTfJMmprDnn5oEHMah (lpMint)
- Offset 496 to 528 is HBFRsHMkaQCJrmXh6CnSwFEAdhNJPAWknHAobcL2t64Q (openOrders)
- Offset 528 to 560 is 6HXQpNkcy9KKkxWi8rhCfxfdLZQuDAj7HjUqRY244mC3 (marketId)
- Offset 560 to 592 is srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX (marketProgramId)
- Offset 592 to 624 is 3jCTzX4pmoyr9Wxe1CiHaM8it3ZHPS3WNdtwAwwD8ZBi (targetOrders)
- Key id not found
- Key baseDecimals not found
- Key quoteDecimals not found
- Key lpDecimals not found
- Key version not found
- Key programId not found
- Key authority not found
- Key marketVersion not found
- Key marketAuthority not found
- Key marketBaseVault not found
- Key marketQuoteVault not found
- Key marketBids not found
- Key marketAsks not found

openbook market data offsets
- Offset 13 to 45 is 6HXQpNkcy9KKkxWi8rhCfxfdLZQuDAj7HjUqRY244mC3 (marketId)
- Offset 53 to 85 is AuqvqC8NhrGMUJaJwUqoYkAnxyQqKZA6tF52ZxmnFTmS (baseMint)
- Offset 85 to 117 is So11111111111111111111111111111111111111112 (quoteMint)
- Offset 117 to 149 is 4UqwHi7PYXqjDQzej9Qu4vfowm4zycwgX2sQwu2tGfR6 (marketBaseVault)
- Offset 165 to 197 is BbXehiDafme9ZYrxpHiPWedtTqBwm2ZgMJGEBvoXmfSu (marketQuoteVault)
- Offset 253 to 285 is BTH36w3BkjaCDiAKzwoRYwtPnEJ4jENHTZTkAhy9EBfk (marketEventQueue)
- Offset 285 to 317 is 7NPj9YNeTH6JxSYn5HRpbjKGnsC95dAx1GehVoTDr8Nz (marketBids)
- Offset 317 to 349 is GSSqqMuSY8s6cxkVaqvKNWDYT1tapfMns8mZ6BKBGwmc (marketAsks)
- Key id not found
- Key lpMint not found
- Key baseDecimals not found
- Key quoteDecimals not found
- Key lpDecimals not found
- Key version not found
- Key programId not found
- Key authority not found
- Key openOrders not found
- Key targetOrders not found
- Key baseVault not found
- Key quoteVault not found
- Key marketVersion not found
- Key marketProgramId not found
- Key marketAuthority not found

Extras:
raydium_pool->lpMint data offsets
- Offset 4 to 36 is 5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1 (authority)

raydium_pool->baseVault data offsets
- Offset 32 to 64 is 5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1 (authority)

openbook_market->market_base_vault data offsets
- Offset 32 to 64 is C1cPxTExbSsjC7xhhgQgD8JndqzgsHSBVdGrzCWo5783 (marketAuthority)

Extras 2:
- baseVault (not the marketBaseVault) points to Authority at [32:64]