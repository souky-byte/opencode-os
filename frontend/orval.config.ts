import { defineConfig } from "orval";

export default defineConfig({
	api: {
		input: {
			target: "http://localhost:3001/api/openapi.json",
		},
		output: {
			mode: "tags-split",
			target: "src/api/generated",
			schemas: "src/api/generated/model",
			client: "react-query",
			clean: true,
			override: {
				mutator: {
					path: "src/api/mutator/custom-instance.ts",
					name: "customInstance",
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
