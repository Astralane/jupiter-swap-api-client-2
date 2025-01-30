use std::env;

use jupiter_swap_api_client::{
    quote::QuoteRequest, swap::SwapRequest, transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey, transaction::VersionedTransaction};
use solana_sdk::{pubkey::Pubkey, signature::NullSigner};
use solana_sdk::signature::{EncodableKey, Keypair, Signer};

const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const NATIVE_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");


#[tokio::main]
async fn main() {
    pub const KEYPAIR_PATH: &str = "/Users/sujithsizon/solana-test-wallet/suj-cli-stake-acc.json";
    let signer = Keypair::read_from_file(KEYPAIR_PATH).unwrap();
    let pubkey = signer.pubkey();

    let api_base_url = env::var("API_BASE_URL").unwrap_or("https://quote-api.jup.ag/v6".into());
    println!("Using base url: {}", api_base_url);

    let jupiter_swap_api_client = JupiterSwapApiClient::new(api_base_url);

    let quote_request = QuoteRequest {
        amount: 5000000,
        input_mint: NATIVE_MINT,
        output_mint: USDC_MINT,
        dexes: Some("Whirlpool,Meteora DLMM,Raydium CLMM".into()),
        slippage_bps: 50,
        ..QuoteRequest::default()
    };

    // GET /quote
    let quote_response = jupiter_swap_api_client.quote(&quote_request).await.unwrap();
    println!("{quote_response:#?}");

    // POST /swap
    let swap_response = jupiter_swap_api_client
        .swap(
            &SwapRequest {
                user_public_key: pubkey,
                quote_response: quote_response.clone(),
                config: TransactionConfig {
                    wrap_and_unwrap_sol: true,
                    compute_unit_price_micro_lamports: Some(jupiter_swap_api_client::transaction_config::ComputeUnitPriceMicroLamports::MicroLamports(100000)),
                    ..TransactionConfig::default()
                },
            },
            None,
        )
        .await
        .unwrap();

    println!("Raw tx len: {}", swap_response.swap_transaction.len());

    let versioned_transaction: VersionedTransaction =
        bincode::deserialize(&swap_response.swap_transaction).unwrap();

    // Replace with a keypair or other struct implementing signer
    let signed_versioned_transaction =
        VersionedTransaction::try_new(versioned_transaction.message, &[&signer]).unwrap();

    // send with rpc client...
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".into());

    // This will fail with "Transaction signature verification failure" as we did not really sign
    let error = rpc_client
        .send_and_confirm_transaction(&signed_versioned_transaction)
        .await
        .unwrap_err();

        
    println!("{error}");

    // POST /swap-instructions
    // let swap_instructions = jupiter_swap_api_client
    //     .swap_instructions(&SwapRequest {
    //         user_public_key: pubkey,
    //         quote_response,
    //         config: TransactionConfig::default(),
    //     })
    //     .await
    //     .unwrap();
    // println!("swap_instructions: {swap_instructions:?}");
}
