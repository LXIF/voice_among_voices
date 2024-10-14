<script lang="ts">
    import {canisterId} from '../../../../declarations/internet_identity';
    import {AuthClient} from '@dfinity/auth-client';

    const handleLogin = async () => {
        console.log(canisterId);
        let iiUrl;
        if (process.env.DFX_NETWORK === 'local') {
            iiUrl = `http://localhost:4943/?canisterId=${canisterId}`;
        } else if (process.env.DFX_NETWORK === 'ic') {
            iiUrl = `https://${canisterId}.ic0.app`;
        } else {
            iiUrl = `https://${canisterId}.dfinity.network`;
        }

        const authClient = await AuthClient.create();

        await authClient.login({
            identityProvider: iiUrl,
            onSuccess: () => {
                console.log('login successful!');
            },
            onError: (e) => {
                console.error(e);
            },
        });

        const identity = authClient.getIdentity();

        console.log(identity);
    };
</script>

<button on:click={handleLogin}>login</button>
