import { defineConfig } from "orval";

export default defineConfig({
	api: {
		input: {
			target: "http://localhost:3001/api/openapi.json",
		},
		output: {
			mode: "tags-split",
			target: "./src/api/generated",
			schemas: "./src/api/generated/model",
			client: "react-query",
			httpClient: "fetch",
			baseUrl: "",
			override: {
				mutator: {
					path: "./src/lib/api-fetcher.ts",
					name: "customFetch",
				},
				query: {
					useQuery: true,
					useMutation: true,
					signal: true,
				},
			},
		},
	},
});
