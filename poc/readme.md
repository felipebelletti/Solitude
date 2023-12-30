Results from https://api.raydium.io/v2/sdk/liquidity/mainnet.json and https://openserum.io/api/serum/token/<token_address> differs on market id.
- https://api.raydium.io/v2/sdk/liquidity/mainnet.json id points to a raydium market id (aka the pool e.g bequiet-SOL) (owner is 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8)
- https://openserum.io/api/serum/token/<token_address> id points to a openbook market id (owner is srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX)

common sizes
- every raydium market data structure has a dataSize (space) of 752

raydium market data offsets
- baseMint  = 400:432
- quoteMint = 432:464
