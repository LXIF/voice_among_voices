<script lang="ts">
    import {AuthClient} from '@dfinity/auth-client';
    import {loginWithInternetIdentity, backend} from '$lib/canisters';


    const handleLogin = async () => {
        console.log(backend);
        await loginWithInternetIdentity();
        console.log(backend);
    };
</script>

<button on:click={handleLogin}>login</button>
