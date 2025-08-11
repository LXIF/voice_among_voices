import type {
    StreamingCallbackToken,
    VoiceNodeIngress,
} from "../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
import { backend } from "./canisters";
import { toastMessage } from "./state/uxState";
import { withRetry } from "$lib/utils/commsUtils";

export const getVoiceNodes = async () => {
    try {
        const nodesResponse = await withRetry(backend.get_voice_nodes, {
            maxRetries: 10,
            delayMs: 150,
            validate: (response) => "Ok" in response,
            onRetry: (attempt) =>
                console.log(`Retrying fetch nodes, attempt ${attempt}...`),
        });
        if ("Err" in nodesResponse) {
            throw nodesResponse.Err;
        }
        return nodesResponse.Ok;
    } catch (error) {
        toastMessage.set("Error fetching nodes, please reload the page.");
        console.error("Error fetching nodes:", error);
        return [];
    }
};

export const getZeroFile = async () => {
    try {
        const response = await withRetry(backend.get_zero_file, {
            maxRetries: 10,
            delayMs: 150,
            validate: (response) => response.status_code === 200,
            onRetry: (attempt) =>
                console.log(`Retrying fetch zero file, attempt ${attempt}...`),
        });
        return response;
    } catch (error) {
        toastMessage.set("Error fetching zero file, please reload the page.");
        console.error("Error fetching zero file:", error);
        return null;
    }
};

export const getAngleFile = async (angle: number) => {
    try {
        let response = await backend.get_angle_file(BigInt(angle), true);

        return response;
    } catch (error) {
        toastMessage.set("Error fetching angle file, please reload the page.");
        console.error("Error fetching angle file:", error);
        return null;
    }
};

export const getAudioParameters = async () => {
    try {
        const response = await withRetry(backend.get_audio_parameters, {
            maxRetries: 10,
            delayMs: 150,
            validate: (response) => response !== null,
            onRetry: (attempt) =>
                console.log(
                    `Retrying fetch audio parameters, attempt ${attempt}...`,
                ),
        });
        return response;
    } catch (error) {
        toastMessage.set(
            "Error fetching audio parameters, please reload the page.",
        );
        console.error("Error fetching audio parameters:", error);
        return null;
    }
};

export const getSimulationParameters = async () => {
    try {
        const response = await withRetry(backend.get_simulation_parameters, {
            maxRetries: 10,
            delayMs: 150,
            validate: (response) => response !== null,
            onRetry: (attempt) =>
                console.log(
                    `Retrying fetch simulation parameters, attempt ${attempt}...`,
                ),
        });
        return response;
    } catch (error) {
        toastMessage.set(
            "Error fetching simulation parameters, please reload the page.",
        );
        console.error("Error fetching simulation parameters:", error);
        return null;
    }
};

export const getColliderCoordinates = async () => {
    try {
        const response = await withRetry(backend.get_collider_coordinates, {
            maxRetries: 10,
            delayMs: 150,
            validate: (response) => response.length > 0,
            onRetry: (attempt) =>
                console.log(
                    `Retrying fetch collider coordinates, attempt ${attempt}...`,
                ),
        });
        return response;
    } catch (error) {
        toastMessage.set(
            "Error fetching collider coordinates, please reload the page.",
        );
        console.error("Error fetching collider coordinates:", error);
        return [];
    }
};

export const getTokenAddress = async () => {
    try {
        const response = await withRetry(backend.get_token_address, {
            maxRetries: 10,
            delayMs: 150,
            validate: (response) => response !== null,
            onRetry: (attempt) =>
                console.log(
                    `Retrying fetch token address, attempt ${attempt}...`,
                ),
        });
        return response;
    } catch (error) {
        toastMessage.set(
            "Error fetching token address, please reload the page.",
        );
        console.error("Error fetching token address:", error);
        return null;
    }
};

export const updateVoiceNode = async (voiceNode: VoiceNodeIngress) => {
    try {
        const response = backend.update_voice_node(voiceNode);
        return response;
    } catch (error) {
        toastMessage.set("Error updating voice node, please reload the page.");
        console.error("Error updating voice node:", error);
        return null;
    }
};

export const getMyPrincipal = async () => {
    try {
        const response = await withRetry(backend.get_my_principal, {
            maxRetries: 10,
            delayMs: 150,
            validate: (response) => response !== null,
            onRetry: (attempt) =>
                console.log(
                    `Retrying fetch my principal, attempt ${attempt}...`,
                ),
        });
        return response;
    } catch (error) {
        toastMessage.set(
            "Error fetching my principal, please reload the page.",
        );
        console.error("Error fetching my principal:", error);
        return null;
    }
};

export const getWalletAddress = async () => {
    try {
        const response = await withRetry(backend.get_wallet_address, {
            maxRetries: 10,
            delayMs: 150,
            validate: (response) => "Ok" in response,
            onRetry: (attempt) =>
                console.log(
                    `Retrying fetch wallet address, attempt ${attempt}...`,
                ),
        });
        return response;
    } catch (error) {
        toastMessage.set(
            "Error fetching wallet address, please reload the page.",
        );
        console.error("Error fetching wallet address:", error);
        return null;
    }
};

export const httpRequestStreamingCallback = async (
    token: StreamingCallbackToken,
) => {
    try {
        const response = await withRetry(
            () => backend.http_request_streaming_callback(token),
            {
                maxRetries: 10,
                delayMs: 150,
                validate: (response) => response !== null,
                onRetry: (attempt) =>
                    console.log(
                        `Retrying http request streaming callback, attempt ${attempt}...`,
                    ),
            },
        );
        return response;
    } catch (error) {
        toastMessage.set(
            "Error fetching http request streaming callback, please reload the page.",
        );
        console.error("Error fetching audio file:", error);
        return null;
    }
};
