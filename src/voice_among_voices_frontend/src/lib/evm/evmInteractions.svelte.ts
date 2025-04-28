import { readContract } from "viem/actions";
import { toastMessage, walletAddress } from "$lib/state/uxState";
import { get } from "svelte/store";
import { wagmiConfig } from "$lib/appKit";
import { writable } from "svelte/store";
import { getTokenAddress } from "$lib/icInteractions";

export let tokenAddress = writable<string | undefined>();

export const fetchTokenAddress = async () => {
    if (!get(tokenAddress)) {
        try {
            const result = await getTokenAddress();
            if (result && result.address) {
                tokenAddress.set(result.address);
            }
        } catch (error) {
            console.error("Error fetching token address:", error);
            toastMessage.set(
                "Error fetching token address, please reload the page.",
            );
        }
    }
    return get(tokenAddress);
};

export const fetchTokens = async () => {
    const nftAddress = await fetchTokenAddress();
    const userAddress = get(walletAddress);

    if (!nftAddress || !userAddress) {
        console.error("Token address or user address not available");
        toastMessage.set("Token address or user address not available");
        return [];
    }

    // ABI for ERC721 contract with specific functions we need
    const abi = [
        {
            inputs: [{ name: "owner", type: "address" }],
            name: "balanceOf",
            outputs: [{ name: "", type: "uint256" }],
            stateMutability: "view",
            type: "function",
        },
        {
            inputs: [
                { name: "owner", type: "address" },
                { name: "index", type: "uint256" },
            ],
            name: "tokenOfOwnerByIndex",
            outputs: [{ name: "", type: "uint256" }],
            stateMutability: "view",
            type: "function",
        },
    ] as const;

    try {
        const config = get(wagmiConfig);
        if (!config) throw "wagmi config not initialized!";
        const client = config.getClient();
        if (!client) throw "wagmi client not initialized!";

        // First, get the balance of tokens owned by the user
        const balance = await readContract(client, {
            address: nftAddress as `0x${string}`,
            abi,
            functionName: "balanceOf",
            args: [userAddress as `0x${string}`],
        });

        if (!balance) return [];

        // Prepare calls to get each token ID
        const tokenPromises = [];
        for (let i = 0; i < Number(balance); i++) {
            tokenPromises.push(
                readContract(client, {
                    address: nftAddress as `0x${string}`,
                    abi,
                    functionName: "tokenOfOwnerByIndex",
                    args: [userAddress as `0x${string}`, BigInt(i)],
                }),
            );
        }

        // Execute all calls and collect results
        const tokenIds = (await Promise.all(tokenPromises)) as bigint[];
        return tokenIds.map((bigNum: bigint) => Number(bigNum));
    } catch (error) {
        console.error("Error fetching tokens:", error);
        toastMessage.set("Error fetching tokens");
        return [];
    }
};
