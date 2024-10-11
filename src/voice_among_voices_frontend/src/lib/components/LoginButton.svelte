<script>
    import {ConnectButton, ConnectDialog, useConnect} from '@connect2ic/svelte';

    const {isConnected, principal, activeProvider} = useConnect({
        onConnect: () => {
            console.log(isConnected, principal, activeProvider);
        },
        onDisconnect: () => {
            // Signed out
        },
    });
</script>

<div>
    <ConnectButton />
    <ConnectDialog />
</div>
