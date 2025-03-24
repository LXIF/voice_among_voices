npm run extract-candid

dfx canister create --all

dfx deploy voice_among_voices_backend --argument $'(
    opt record {
        siwe_canister_principal = opt principal "'$(dfx canister id ic_siwe_provider)'";
        token_address = opt "0x1d0406e0df3f50a1399d299c28e58f8444508013";
        dev_mode = opt true;
    }
)'

dfx deploy voice_among_voices_frontend

# dfx deploy evm_rpc --argument '(record {})'
dfx deps pull
dfx deps init evm_rpc --argument '(record {})'
dfx deps deploy

dfx deploy ic_siwe_provider --argument $'(
    record {
        domain = "localhost";
        uri = "http://127.0.0.1:5173";
        salt = "my-secret-salt";
        chain_id = opt 1;
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