import ICRC7 "mo:icrc7-mo";

module {
    public let defaultConfig = func(caller : Principal) : ICRC7.InitArgs {
        ?{
            symbol = ?"VAV";
            name = ?"Voice Among Voices";
            description = ?"A voice on the map and an angle to listen";
            logo = ?""; //TODO
            supply_cap = ?360;
            allow_transfers = ?true;
            max_query_batch_size = ?100;
            max_update_batch_size = ?100;
            default_take_value = ?1000;
            max_take_value = ?10000;
            max_memo_size = ?512;
            permitted_drift = null;
            tx_window = null;
            burn_account = null; //burned nfts are deleted
            deployer = caller; // TODO: here we would input our main canister
            supported_standards = null;
        };
    };
};
