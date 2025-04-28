import { toastMessage } from "$lib/state/uxState";

export async function withRetry<T>(
    fn: () => Promise<T>,
    options: {
        maxRetries?: number;
        delayMs?: number;
        validate?: (result: T) => boolean;
        onRetry?: (attempt: number, error?: any) => void;
    } = {},
): Promise<T> {
    const {
        maxRetries = 3,
        delayMs = 1000,
        validate = (result) => result != null,
        onRetry = (attempt, error) =>
            console.log(`Attempt ${attempt} failed:`, error),
    } = options;

    let lastError: any;

    for (let attempt = 0; attempt < maxRetries; attempt++) {
        try {
            const result = await fn();
            if (validate(result)) {
                return result;
            }
            lastError = new Error("Validation failed");
        } catch (error) {
            // toastMessage.set("Error fetching data");
            lastError = error;
        }

        if (attempt < maxRetries - 1) {
            onRetry(attempt + 1, lastError);
            // Exponential backoff: delayMs * 2^attempt
            const backoffDelay = delayMs * Math.pow(2, attempt);
            await new Promise((resolve) => setTimeout(resolve, backoffDelay));
        }
    }

    throw lastError || new Error(`Failed after ${maxRetries} attempts`);
}
