use std::str::FromStr;
use subxt_signer::sr25519::Keypair;
use subxt::utils::AccountId32;

pub fn get_signer() -> Keypair {
    // Load the .env file contents into the environment
    dotenv::dotenv().ok();
    use std::env;
    
    let mut secret_seed = env::var("SECRET_SEED").expect("SECRET_SEED must be set in .env.");
    // println!("secret seed: {secret_seed:?}");
    if secret_seed.starts_with("0x") { secret_seed = (&secret_seed[2..]).to_string(); }

    // Decode the hex string into a vector of bytes
    let bytes: Vec<u8> = hex::decode(secret_seed).expect("Decoding failed");
    
    // Convert the vector to a fixed-size array
    let array: [u8; 32] = bytes.try_into().expect("Expected a Vec of length 32");
    
    // Assuming you have your seed as a [u8; 32]
    // let seed: [u8; 32] = [0; 32]; // Example seed, replace with your actual seed
    // let pair = Pair::from_seed(&seed);
    let pair = Keypair::from_secret_key(array).expect("Must provide a valid signer secret seed.");
    println!("pair retrieved");
    // let public: Public = pair.public();
    pair
}

pub fn to_account(account: &str) -> AccountId32 {
    let account_id = AccountId32::from_str(account).unwrap();

    account_id
}
