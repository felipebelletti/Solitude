const web3 = require('@solana/web3.js');
const { PublicKey } = web3;

/*
Test and real data:
Token: Etpju1XBPwjSgQ5tcdPEKsLZwmwmGNonUbacUBjMf2gF
BaseMint: Etpju1XBPwjSgQ5tcdPEKsLZwmwmGNonUbacUBjMf2gF
QuoteMint: So11111111111111111111111111111111111111112

{"id":"8CTgrTG9qe4iK4iKGyidbUii7MYS4uzX2gmE2bXDa6WV","baseMint":"Etpju1XBPwjSgQ5tcdPEKsLZwmwmGNonUbacUBjMf2gF","quoteMint":"So11111111111111111111111111111111111111112","lpMint":"DitUDr63iJTEVpAN8oX1mJyLGH4QwqFKoUuqp5xb4xLA","baseDecimals":6,"quoteDecimals":9,"lpDecimals":6,"version":4,"programId":"675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8","authority":"5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1","openOrders":"C8nu3jebcFscEUe4jHBSKMUBYeab8jmDo54CTwYFc9gE","targetOrders":"5VeCmU9nj8zbFMTyVvRKGnKrR1HuzNLAVqZguyFipvQJ","baseVault":"GbbqncbV7cxphfuXsYMYrsvd7HTX9qJx23Jmw5sEjz47","quoteVault":"jAQD81qSGJUGczhh2Vdnvxj68SyyezJ8giGG4zCCDFi","withdrawQueue":"11111111111111111111111111111111","lpVault":"11111111111111111111111111111111","marketVersion":4,"marketProgramId":"srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX","marketId":"2ozU23yywPR4uhTKQwUUMp936rPtLdxc4Fsdy6LibPWH","marketAuthority":"38MoDrijmybQFeZikMGegZ2bTBzASbG34peztbkjcEPx","marketBaseVault":"FGoMpWDmxRXh4SiMFkUichRow6Z58215AwhVNFZ4uLLh","marketQuoteVault":"HaSmnKmukZeBKyD6dhwfUDL5oTJAEza1aSH1BjYSrSkt","marketBids":"NgtgsLU19YDHJm8QD79DDC9sVBTA28wjUrjNc8jQEmL","marketAsks":"J4waFJhwAJ7P21ECHquxZDQCesFiBy4t3u2qMNVdrEPp","marketEventQueue":"5k1nfotLp4W1neE2wkZgsLrQFJUcoUoSfouRx3W1RfGp"},
*/

// Define the connection to the Solana cluster
const connection = new web3.Connection("https://ancient-chaotic-aura.solana-mainnet.quiknode.pro/316c14d6ae6c6f4a358f0a44b3e34244e6e34783");

// const yourTestMarketAddress = new PublicKey('8CTgrTG9qe4iK4iKGyidbUii7MYS4uzX2gmE2bXDa6WV'); // Replace with actual test market address
// const baseTokenMintString = 'Etpju1XBPwjSgQ5tcdPEKsLZwmwmGNonUbacUBjMf2gF'; // Replace with actual base token mint
// const quoteTokenMintString = 'So11111111111111111111111111111111111111112'; // Replace with actual quote token mint

const yourTestMarketAddress = new PublicKey('Ho9rEHJoaB3T9j6Anv2AH75ftUsLrJWvn4ffEN3GiYBn'); // Replace with actual test market address
const baseTokenMintString = 'AuqvqC8NhrGMUJaJwUqoYkAnxyQqKZA6tF52ZxmnFTmS'; // Replace with actual base token mint
const quoteTokenMintString = 'So11111111111111111111111111111111111111112'; // Replace with actual quote token mint

async function findMarkets() {
    // Array of token mints to compare
    const tokens = [baseTokenMintString.toLowerCase(), quoteTokenMintString.toLowerCase()];

    // Get account information for the test market
    const market = await connection.getAccountInfo(yourTestMarketAddress);
    console.log(market);

    // Initialize offsets
    let x = 0;
    let y = 32;

    // Find offsets
    while (y <= market.data.length) {
        let sliced = new PublicKey(market.data.subarray(x, y)).toString();
        if (tokens.includes(sliced.toLowerCase())) {
            console.log('Found token at offsets:', sliced, x, y);
        }
        x++;
        y++;
    }

    // Using the offsets found, query for accounts
    // Note: You need to determine the filters based on the offsets and the specific mint you are looking for
    // const openbookProgram = new PublicKey('...'); // Replace with actual OpenBook program ID
    // const filters = [{ dataSize: ... }]; // Define your filters here

    // const accounts = await connection.getProgramAccounts(openbookProgram, { filters: filters });
    
    // // Process the found accounts
    // accounts.forEach((account: any) => {
    //     // Process each account as required
    //     console.log('Found market account:', account.pubkey.toString());
    // });
}

findMarkets().catch(err => console.error(err));

/*
Pending testing, concept should work:

// ... [Previous code initialization]

const dataSize = 752; // Data size of market accounts
const baseMintOffset = 400; // The offset for the base mint address

const baseMintPublicKey = new PublicKey(baseTokenMintString);

// Filters for the getProgramAccounts call
const filters = [
    { dataSize: dataSize },
    {
        memcmp: {
            offset: baseMintOffset,
            bytes: baseMintPublicKey.toBase58(),
        },
    },
];

const marketAccounts = await connection.getProgramAccounts(openbookProgram, { filters: filters });

marketAccounts.forEach(account => {
    // Process each account to extract the complete market object
    console.log('Found market account:', account.pubkey.toString());
    // Additional processing as needed
});

// ... [Any additional code]
*/