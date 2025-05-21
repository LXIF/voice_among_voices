npm run extract-candid
npm run prebuild
npm run build

dfx canister create --all --ic --identity LXIF-private

# OLD on Sepolia:
# dfx deploy voice_among_voices_backend --ic --identity LXIF-private --argument $'(
#     opt record {
#         siwe_canister_principal = opt principal "'$(dfx canister id ic_siwe_provider --ic)'";
#         token_address = opt "0x1d0406e0df3f50a1399d299c28e58f8444508013";
#         dev_mode = opt false;
#     }
# )'

# NEW on Base Mainnet:
dfx deploy voice_among_voices_backend --ic --identity LXIF-private --argument $'(
    opt record {
        siwe_canister_principal = opt principal "'$(dfx canister id ic_siwe_provider --ic)'";
        token_address = opt "0xB6AeC30a1252C71De5b14bB40C5339Bd0B80fc13";
        dev_mode = opt true;
        admin_token_id = opt 360;
        token_buy_link = opt "https://basescan.org/address/0xB6AeC30a1252C71De5b14bB40C5339Bd0B80fc13#writeContract#F2";
    }
)'

# dfx deploy evm_rpc --argument '(record {})'
dfx deps pull --ic --identity LXIF-private
dfx deps init evm_rpc --argument '(record {})' --ic --identity LXIF-private
dfx deps deploy --ic --identity LXIF-private

dfx deploy ic_siwe_provider --ic --identity LXIF-private --argument $'(
    record {
        domain = "'$(dfx canister id voice_among_voices_frontend --ic)'.icp0.io";
        uri = "https://'$(dfx canister id voice_among_voices_frontend --ic)'.icp0.io";
        salt = "my-secret-salt";
        chain_id = opt 8453;
        scheme = opt "http";
        statement = opt "Login to Voice Among Voices";
        sign_in_expires_in = opt 300000000000;
        session_expires_in = opt 604800000000000;
        targets = opt vec {
            "'$(dfx canister id ic_siwe_provider --ic)'";
            "'$(dfx canister id voice_among_voices_backend --ic)'";
            "'$(dfx canister id voice_among_voices_frontend --ic)'";
        };
    }
)'

dfx deploy voice_among_voices_frontend --ic --identity LXIF-private

#dfx deploy