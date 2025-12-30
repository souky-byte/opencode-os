"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import type {
	ClientMessage,
	ServerMessage,
	Event,
	EventEnvelope,
	SubscriptionFilter,
} from "~/types/generated";

const WS_URL = process.env.NEXT_PUBLIC_WS_URL ?? "ws://localhost:3001/ws";

type ConnectionState = "connecting" | "connected" | "disconnected" | "error";

interface UseStudioWebSocketOptions {
	autoConnect?: boolean;
	taskIds?: string[];
	onEvent?: (event: Event, envelope: EventEnvelope) => void;
	onConnectionChange?: (state: ConnectionState) => void;
	reconnectInterval?: number;
	maxReconnectAttempts?: number;
}

interface UseStudioWebSocketReturn {
	connectionState: ConnectionState;
	connect: () => void;
	disconnect: () => void;
	subscribe: (filter?: SubscriptionFilter) => void;
	unsubscribe: () => void;
	isSubscribed: boolean;
}

export function useStudioWebSocket(
	options: UseStudioWebSocketOptions = {},
): UseStudioWebSocketReturn {
	const {
		autoConnect = true,
		taskIds,
		onEvent,
		onConnectionChange,
		reconnectInterval = 3000,
		maxReconnectAttempts = 5,
	} = options;

	const [connectionState, setConnectionState] = useState<ConnectionState>("disconnected");
	const [isSubscribed, setIsSubscribed] = useState(false);

	const wsRef = useRef<WebSocket | null>(null);
	const reconnectAttemptsRef = useRef(0);
	const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
	const pingIntervalRef = useRef<NodeJS.Timeout | null>(null);

	const updateConnectionState = useCallback(
		(state: ConnectionState) => {
			setConnectionState(state);
			onConnectionChange?.(state);
		},
		[onConnectionChange],
	);

	const sendMessage = useCallback((message: ClientMessage) => {
		if (wsRef.current?.readyState === WebSocket.OPEN) {
			wsRef.current.send(JSON.stringify(message));
		}
	}, []);

	const subscribe = useCallback(
		(filter?: SubscriptionFilter) => {
			const effectiveFilter: SubscriptionFilter | null =
				filter ?? (taskIds ? { task_ids: taskIds } : null);

			sendMessage({ type: "subscribe", filter: effectiveFilter });
		},
		[sendMessage, taskIds],
	);

	const unsubscribe = useCallback(() => {
		sendMessage({ type: "unsubscribe" });
		setIsSubscribed(false);
	}, [sendMessage]);

	const startPingInterval = useCallback(() => {
		pingIntervalRef.current = setInterval(() => {
			sendMessage({ type: "ping" });
		}, 30000);
	}, [sendMessage]);

	const stopPingInterval = useCallback(() => {
		if (pingIntervalRef.current) {
			clearInterval(pingIntervalRef.current);
			pingIntervalRef.current = null;
		}
	}, []);

	const handleMessage = useCallback(
		(event: MessageEvent) => {
			try {
				const message = JSON.parse(event.data) as ServerMessage;

				switch (message.type) {
					case "event":
						onEvent?.(message.envelope.event, message.envelope);
						break;
					case "subscribed":
						setIsSubscribed(true);
						break;
					case "unsubscribed":
						setIsSubscribed(false);
						break;
					case "pong":
						break;
					case "error":
						console.error("[WebSocket] Server error:", message.message);
						break;
				}
			} catch (e) {
				console.error("[WebSocket] Failed to parse message:", e);
			}
		},
		[onEvent],
	);

	const connect = useCallback(() => {
		if (wsRef.current?.readyState === WebSocket.OPEN) {
			return;
		}

		if (reconnectTimeoutRef.current) {
			clearTimeout(reconnectTimeoutRef.current);
			reconnectTimeoutRef.current = null;
		}

		updateConnectionState("connecting");

		try {
			const ws = new WebSocket(WS_URL);
			wsRef.current = ws;

			ws.onopen = () => {
				reconnectAttemptsRef.current = 0;
				updateConnectionState("connected");
				startPingInterval();
				subscribe();
			};

			ws.onmessage = handleMessage;

			ws.onclose = () => {
				stopPingInterval();
				setIsSubscribed(false);
				updateConnectionState("disconnected");

				if (reconnectAttemptsRef.current < maxReconnectAttempts) {
					const delay = reconnectInterval * Math.pow(2, reconnectAttemptsRef.current);
					reconnectAttemptsRef.current++;

					reconnectTimeoutRef.current = setTimeout(() => {
						connect();
					}, delay);
				}
			};

			ws.onerror = () => {
				updateConnectionState("error");
			};
		} catch (e) {
			console.error("[WebSocket] Connection error:", e);
			updateConnectionState("error");
		}
	}, [
		updateConnectionState,
		startPingInterval,
		stopPingInterval,
		subscribe,
		handleMessage,
		reconnectInterval,
		maxReconnectAttempts,
	]);

	const disconnect = useCallback(() => {
		reconnectAttemptsRef.current = maxReconnectAttempts;

		if (reconnectTimeoutRef.current) {
			clearTimeout(reconnectTimeoutRef.current);
			reconnectTimeoutRef.current = null;
		}

		stopPingInterval();

		if (wsRef.current) {
			wsRef.current.close();
			wsRef.current = null;
		}

		updateConnectionState("disconnected");
	}, [stopPingInterval, updateConnectionState, maxReconnectAttempts]);

	useEffect(() => {
		if (autoConnect) {
			connect();
		}

		return () => {
			disconnect();
		};
	}, [autoConnect, connect, disconnect]);

	useEffect(() => {
		if (isSubscribed && taskIds) {
			subscribe({ task_ids: taskIds });
		}
	}, [taskIds, isSubscribed, subscribe]);

	return {
		connectionState,
		connect,
		disconnect,
		subscribe,
		unsubscribe,
		isSubscribed,
	};
}
