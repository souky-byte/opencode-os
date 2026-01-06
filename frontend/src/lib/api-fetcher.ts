export async function customFetch<T>(url: string, options?: RequestInit): Promise<T> {
	const response = await fetch(url, {
		...options,
		headers: {
			"Content-Type": "application/json",
			...options?.headers,
		},
	});

	const status = response.status;
	const headers = response.headers;

	if (!response.ok) {
		const error = await response.text().catch(() => response.statusText);
		throw new Error(`API Error ${status}: ${error}`);
	}

	if (status === 204 || status === 202) {
		return { data: undefined, status, headers } as T;
	}

	const data = await response.json();
	return { data, status, headers } as T;
}

export type CustomFetchError = Error;
