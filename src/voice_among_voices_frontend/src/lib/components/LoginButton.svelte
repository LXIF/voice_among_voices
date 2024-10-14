<script lang="ts">
    import {canisterId} from '../../../../declarations/internet_identity';
    import {AuthClient} from '@dfinity/auth-client';
    import {loginWithInternetIdentity, backend} from '$lib/canisters';
    // import {HttpAgent} from '@dfinity/agent';
    // import {
    //     createActor,
    //     voice_among_voices_backend,
    // } from '../../../../declarations/voice_among_voices_backend';

    const handleLogin = async () => {
        console.log(backend);
        await loginWithInternetIdentity();
        console.log(backend);
    };
</script>

<button on:click={handleLogin}>login</button>
