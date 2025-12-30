/**
 * Custom Axios-like instance for Orval generated API client.
 * Uses native fetch with configurable base URL.
 */

const BASE_URL = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3001";

export type CustomInstanceParams<T> = {
	url: string;
	method: "GET" | "POST" | "PUT" | "DELETE" | "PATCH";
	params?: Record<string, string>;
	headers?: Record<string, string>;
	data?: T;
	signal?: AbortSignal;
};

export const customInstance = async <T>({
	url,
	method,
	params,
	headers,
	data,
	signal,
}: CustomInstanceParams<unknown>): Promise<T> => {
	const searchParams = params ? `?${new URLSearchParams(params).toString()}` : "";

	const response = await fetch(`${BASE_URL}${url}${searchParams}`, {
		method,
		headers: {
			"Content-Type": "application/json",
			...headers,
		},
		body: data ? JSON.stringify(data) : undefined,
		signal,
	});

	if (!response.ok) {
		const error = await response.text();
		throw new Error(error || `HTTP ${response.status}`);
	}

	// Handle empty responses (204 No Content)
	const contentType = response.headers.get("content-type");
	if (response.status === 204 || !contentType?.includes("application/json")) {
		return undefined as T;
	}

	return response.json() as Promise<T>;
};

export default customInstance;
