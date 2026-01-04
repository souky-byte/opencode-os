import { useEffect, useRef } from "react";
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
					}`}
				>
					<p className="text-sm whitespace-pre-wrap">{message.content}</p>
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
