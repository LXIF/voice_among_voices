dfx deploy voice_among_voices_backend --ic --identity LXIF-private --argument $'(
    opt record {
        siwe_canister_principal = null;
        token_address = opt "0xb32CEf004a828F0E2f87c6b188593f9cEd8FD01D";
        dev_mode = null;
        admin_token_id = null;
        token_buy_link = opt "https://opensea.io/item/base/0xb32CEf004a828F0E2f87c6b188593f9cEd8FD01D/";
    }
)'