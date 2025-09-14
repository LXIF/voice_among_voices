npm run extract-candid

dfx deps pull
dfx deps init evm_rpc --argument '(record {})'
dfx deps deploy

dfx canister create --all

# OLD on Sepolia
# dfx deploy voice_among_voices_backend --argument $'(
#     opt record {
#         siwe_canister_principal = opt principal "'$(dfx canister id ic_siwe_provider)'";
#         token_address = opt "0x1d0406e0df3f50a1399d299c28e58f8444508013";
#         dev_mode = opt true;
#     }
# )'

# NEW on Base Mainnet
dfx deploy voice_among_voices_backend --argument $'(
    opt record {
        siwe_canister_principal = opt principal "'$(dfx canister id ic_siwe_provider)'";
        token_address = opt "0xb32CEf004a828F0E2f87c6b188593f9cEd8FD01D";
        dev_mode = opt true;
        admin_token_id = opt 0;
        token_buy_link = opt "https://opensea.io/item/base/0xb32CEf004a828F0E2f87c6b188593f9cEd8FD01D";
    }
)'

dfx deploy voice_among_voices_frontend


dfx deploy ic_siwe_provider --argument $'(
    record {
        domain = "localhost";
        uri = "http://127.0.0.1:5173";
        salt = "my-secret-salt";
        chain_id = opt 8453;
        scheme = opt "http";
        statement = opt "Login to the app";
        sign_in_expires_in = opt 300000000000;
        session_expires_in = opt 604800000000000;
        targets = opt vec {
            "'$(dfx canister id ic_siwe_provider)'";
            "'$(dfx canister id voice_among_voices_backend)'";
            "'$(dfx canister id voice_among_voices_frontend)'";
        };
    }
)'

#dfx deploy

npm run prebuild