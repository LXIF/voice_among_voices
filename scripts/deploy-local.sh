npm run extract-candid

dfx canister create --all

dfx deploy voice_among_voices_backend --argument $'(
    opt record {
        siwe_canister_principal = opt principal "'$(dfx canister id ic_siwe_provider)'";
        token_address = opt "0x431352373f5d99b8d388bce95d995fab762554e1";
    }
)'

dfx deploy voice_among_voices_frontend

dfx deploy evm_rpc --argument '(record {})'

dfx deploy ic_siwe_provider --argument $'(
    record {
        domain = "127.0.0.1";
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

dfx deploy

npm run prebuild