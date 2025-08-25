dfx deploy voice_among_voices_backend --ic --identity LXIF-private --argument $'(
    opt record {
        siwe_canister_principal = null;
        token_address = opt "TOKEN";
        dev_mode = null;
        admin_token_id = null;
        token_buy_link = opt "https://opensea.io/item/base/TOKEN/";
    }
)'