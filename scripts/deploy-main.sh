npm run extract-candid
npm run prebuild
npm run build

dfx canister create --all --ic --identity LXIF-private

dfx deploy voice_among_voices_backend --argument $'(
    opt record {
        siwe_canister_principal = opt principal "'$(dfx canister id ic_siwe_provider --ic)'";
        token_address = opt "0x1d0406e0df3f50a1399d299c28e58f8444508013";
        dev_mode = opt false;
    }
)' --ic --identity LXIF-private

# dfx deploy evm_rpc --argument '(record {})'
dfx deps pull --ic --identity LXIF-private
dfx deps init evm_rpc --argument '(record {})' --ic --identity LXIF-private
dfx deps deploy --ic --identity LXIF-private

dfx deploy ic_siwe_provider --argument $'(
    record {
        domain = "localhost";
        uri = "'$(dfx canister id voice_among_voices_frontend --ic)'.icp0.io";
        salt = "my-secret-salt";
        chain_id = opt 11155111;
        scheme = opt "http";
        statement = opt "Login to voice among voices";
        sign_in_expires_in = opt 300000000000;
        session_expires_in = opt 604800000000000;
        targets = opt vec {
            "'$(dfx canister id ic_siwe_provider --ic)'";
            "'$(dfx canister id voice_among_voices_backend --ic)'";
            "'$(dfx canister id voice_among_voices_frontend --ic)'";
        };
    }
)' --ic --identity LXIF-private

dfx deploy voice_among_voices_frontend --ic --identity LXIF-private

#dfx deploy