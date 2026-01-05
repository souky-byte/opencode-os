import { useEffect, useRef } from "react";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { vscDarkPlus } from "react-syntax-highlighter/dist/esm/styles/prism";
import { useAskWiki } from "@/api/generated/wiki/wiki";
import { type ChatMessage, useWikiStore } from "@/stores/useWikiStore";

export function WikiChat() {
	const {
		chatMessages,
		chatInput,
		isChatLoading,
		conversationId,
		addChatMessage,
		setChatInput,
		setIsChatLoading,
		setConversationId,
		clearChat,
	} = useWikiStore();

	const messagesEndRef = useRef<HTMLDivElement>(null);
	const inputRef = useRef<HTMLTextAreaElement>(null);
	const askMutation = useAskWiki();

	// Auto-scroll to bottom when new messages arrive
	useEffect(() => {
		messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
	}, [chatMessages]);

	// Focus input on mount
	useEffect(() => {
		inputRef.current?.focus();
	}, []);

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		const question = chatInput.trim();
		if (!question || isChatLoading) return;

		// Add user message
		const userMessage: ChatMessage = { role: "user", content: question };
		addChatMessage(userMessage);
		setChatInput("");
		setIsChatLoading(true);

		try {
			const result = await askMutation.mutateAsync({
				data: {
					question,
					conversation_id: conversationId ?? undefined,
				},
			});

			if (result.status === 200) {
				const { answer, conversation_id: newConversationId, sources } = result.data;

				if (newConversationId && !conversationId) {
					setConversationId(newConversationId);
				}

				const assistantMessage: ChatMessage = {
					role: "assistant",
					content: answer,
					sources: sources?.map((s) => ({
						file_path: s.file_path,
						start_line: s.start_line,
						end_line: s.end_line,
						score: s.score,
						snippet: s.snippet,
					})),
				};
				addChatMessage(assistantMessage);
			} else {
				addChatMessage({
					role: "assistant",
					content: "Sorry, I received an unexpected response from the server. Please try again.",
				});
			}
		} catch (error) {
			// Add error message
			addChatMessage({
				role: "assistant",
				content: "Sorry, I encountered an error while processing your question. Please try again.",
			});
		} finally {
			setIsChatLoading(false);
		}
	};

	const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
		if (e.key === "Enter" && !e.shiftKey) {
			e.preventDefault();
			void handleSubmit(e);
		}
	};

	return (
		<div className="flex flex-col h-full max-w-4xl mx-auto">
			{/* Header */}
			<div className="flex items-center justify-between px-6 py-4 border-b border-border">
				<div>
					<h2 className="text-xl font-semibold">Ask About Your Codebase</h2>
					<p className="text-sm text-muted-foreground">
						Ask questions and get answers with source references
					</p>
				</div>
				{chatMessages.length > 0 && (
					<button
						type="button"
						onClick={clearChat}
						className="text-sm text-muted-foreground hover:text-foreground transition-colors"
					>
						Clear chat
					</button>
				)}
			</div>

			{/* Messages */}
			<div className="flex-1 overflow-y-auto p-6 space-y-6">
				{chatMessages.length === 0 ? (
					<div className="flex flex-col items-center justify-center h-full text-center">
						<svg
							className="h-16 w-16 text-muted-foreground mb-4"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							strokeWidth="1"
						>
							<path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" />
						</svg>
						<h3 className="text-lg font-medium mb-2">Start a conversation</h3>
						<p className="text-sm text-muted-foreground max-w-md">
							Ask questions about your codebase. I'll search through the indexed code and provide
							answers with source references.
						</p>
						<div className="mt-6 flex flex-wrap gap-2 justify-center">
							{[
								"How is authentication implemented?",
								"What are the main API endpoints?",
								"Explain the database schema",
								"Where is error handling done?",
							].map((suggestion) => (
								<button
									key={suggestion}
									type="button"
									onClick={() => setChatInput(suggestion)}
									className="text-xs px-3 py-1.5 rounded-full border border-border hover:bg-accent transition-colors"
								>
									{suggestion}
								</button>
							))}
						</div>
					</div>
				) : (
					<>
						{chatMessages.map((message, index) => (
							<ChatMessageItem key={`msg-${index}`} message={message} />
						))}
						{isChatLoading && (
							<div className="flex items-center gap-2 text-muted-foreground">
								<div className="flex gap-1">
									<span className="w-2 h-2 rounded-full bg-primary animate-bounce" />
									<span
										className="w-2 h-2 rounded-full bg-primary animate-bounce"
										style={{ animationDelay: "0.1s" }}
									/>
									<span
										className="w-2 h-2 rounded-full bg-primary animate-bounce"
										style={{ animationDelay: "0.2s" }}
									/>
								</div>
								<span className="text-sm">Thinking...</span>
							</div>
						)}
						<div ref={messagesEndRef} />
					</>
				)}
			</div>

			{/* Input */}
			<div className="border-t border-border p-4">
				<form onSubmit={handleSubmit} className="flex gap-2">
					<textarea
						ref={inputRef}
						value={chatInput}
						onChange={(e) => setChatInput(e.target.value)}
						onKeyDown={handleKeyDown}
						placeholder="Ask a question about your code..."
						rows={1}
						className="flex-1 px-4 py-2 bg-accent border border-border rounded-md resize-none focus:outline-none focus:ring-2 focus:ring-primary/50"
					/>
					<button
						type="submit"
						disabled={isChatLoading || !chatInput.trim()}
						className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
					>
						<svg
							className="h-5 w-5"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							strokeWidth="2"
						>
							<path d="M22 2L11 13" />
							<path d="M22 2L15 22L11 13L2 9L22 2Z" />
						</svg>
					</button>
				</form>
			</div>
		</div>
	);
}

function ChatMessageItem({ message }: { message: ChatMessage }) {
	const isUser = message.role === "user";

	return (
		<div className={`flex ${isUser ? "justify-end" : "justify-start"}`}>
			<div className={`max-w-[80%] ${isUser ? "order-2" : "order-1"}`}>
				{/* Message bubble */}
				<div
					className={`rounded-lg px-4 py-2 ${
						isUser ? "bg-primary text-primary-foreground" : "bg-accent"
					} overflow-hidden`}
				>
					{isUser ? (
						<p className="text-sm whitespace-pre-wrap">{message.content}</p>
					) : (
						<div className="text-sm">
							<Markdown
								remarkPlugins={[remarkGfm]}
								rehypePlugins={[rehypeRaw]}
								components={{
									h1: ({ node, ...props }) => (
										<h1 className="text-lg font-bold mt-4 mb-2" {...props} />
									),
									h2: ({ node, ...props }) => (
										<h2 className="text-base font-semibold mt-3 mb-2" {...props} />
									),
									h3: ({ node, ...props }) => (
										<h3 className="text-sm font-semibold mt-2 mb-1" {...props} />
									),
									p: ({ node, ...props }) => (
										<p className="mb-2 last:mb-0 leading-relaxed" {...props} />
									),
									a: ({ node, ...props }) => (
										<a className="text-primary hover:underline font-medium underline" {...props} />
									),
									ul: ({ node, ...props }) => (
										<ul className="list-disc ml-4 mb-2 space-y-1" {...props} />
									),
									ol: ({ node, ...props }) => (
										<ol className="list-decimal ml-4 mb-2 space-y-1" {...props} />
									),
									li: ({ node, ...props }) => <li className="pl-1" {...props} />,
									blockquote: ({ node, ...props }) => (
										<blockquote
											className="border-l-2 border-primary/20 pl-3 italic my-2 text-muted-foreground"
											{...props}
										/>
									),
									details: ({ node, ...props }) => (
										<details
											className="my-2 rounded border border-border bg-background/30 overflow-hidden"
											{...props}
										/>
									),
									summary: ({ node, ...props }) => (
										<summary
											className="px-3 py-1.5 cursor-pointer font-medium bg-muted/50 hover:bg-muted/70 transition-colors select-none text-sm"
											{...props}
										/>
									),
									table: ({ node, ...props }) => (
										<div className="my-2 w-full overflow-y-auto rounded border border-border bg-card">
											<table className="w-full text-xs" {...props} />
										</div>
									),
									thead: ({ node, ...props }) => <thead className="bg-muted/50" {...props} />,
									tbody: ({ node, ...props }) => (
										<tbody
											className="divide-y divide-border [&>tr:nth-child(even)]:bg-muted/30"
											{...props}
										/>
									),
									tr: ({ node, ...props }) => (
										<tr className="transition-colors hover:bg-muted/50" {...props} />
									),
									th: ({ node, ...props }) => (
										<th
											className="px-2 py-1 text-left align-middle font-medium text-muted-foreground"
											{...props}
										/>
									),
									td: ({ node, ...props }) => <td className="p-2 align-middle" {...props} />,
									code: ({ node, inline, className, children, ...props }: any) => {
										const match = /language-(\w+)/.exec(className || "");
										return !inline && match ? (
											<div className="my-2 rounded overflow-hidden">
												<SyntaxHighlighter
													style={vscDarkPlus}
													language={match[1]}
													PreTag="div"
													customStyle={{ margin: 0, borderRadius: 0, fontSize: "0.75rem" }}
													{...props}
												>
													{String(children).replace(/\n$/, "")}
												</SyntaxHighlighter>
											</div>
										) : (
											<code
												className="bg-background/50 px-1 py-0.5 rounded font-mono text-foreground"
												{...props}
											>
												{children}
											</code>
										);
									},
								}}
							>
								{message.content}
							</Markdown>
						</div>
					)}
				</div>

				{/* Sources (for assistant messages) */}
				{!isUser && message.sources && message.sources.length > 0 && (
					<div className="mt-2 space-y-1">
						<p className="text-xs text-muted-foreground">Sources:</p>
						{message.sources.map((source, index) => (
							<details
								key={`source-${index}`}
								className="text-xs border border-border rounded overflow-hidden"
							>
								<summary className="px-2 py-1 bg-accent/50 cursor-pointer hover:bg-accent transition-colors">
									<span className="font-mono">{source.file_path}</span>
									<span className="text-muted-foreground ml-1">
										:{source.start_line}-{source.end_line}
									</span>
									<span className="text-muted-foreground ml-2">
										({Math.round(source.score * 100)}% match)
									</span>
								</summary>
								<pre className="p-2 overflow-x-auto text-xs bg-card">
									<code>{source.snippet}</code>
								</pre>
							</details>
						))}
					</div>
				)}
			</div>
		</div>
	);
}
