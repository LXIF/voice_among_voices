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
        token_address = opt "0xb32CEf004a828F0E2f87c6b188593f9cEd8FD01D";
        dev_mode = opt false;
        admin_token_id = opt 0;
        token_buy_link = opt "https://opensea.io/item/base/0xb32CEf004a828F0E2f87c6b188593f9cEd8FD01D";
    }
)'

# dfx deploy evm_rpc --argument '(record {})'
# dfx deps pull --ic --identity LXIF-private
# dfx deps init evm_rpc --argument '(record {})' --ic --identity LXIF-private
# dfx deps deploy --ic --identity LXIF-private

dfx deploy ic_siwe_provider --ic --identity LXIF-private --argument $'(
    record {
        domain = "voiceamongvoic.es";
        uri = "https://voiceamongvoic.es";
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