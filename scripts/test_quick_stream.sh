#!/bin/bash

# Quick streaming test - just planning phase
# Usage: ./test_quick_stream.sh [port]

BASE_URL="http://localhost:${1:-3001}"

echo "=== Quick Streaming Test ==="
echo "Server: $BASE_URL"
echo ""

# Create task
echo "[$(date +%H:%M:%S.%3N)] Creating task..."
TASK=$(curl -s -X POST "$BASE_URL/api/tasks" \
    -H "Content-Type: application/json" \
    -d '{"title":"Quick stream test","description":"Testing live streaming"}')

TASK_ID=$(echo "$TASK" | jq -r '.id')
echo "[$(date +%H:%M:%S.%3N)] Task ID: $TASK_ID"

# Start planning (triggers execution)
echo "[$(date +%H:%M:%S.%3N)] Starting planning..."
TRANSITION=$(curl -s -X POST "$BASE_URL/api/tasks/$TASK_ID/transition" \
    -H "Content-Type: application/json" \
    -d '{"status":"planning"}')

# Wait for session to be created
sleep 0.3

# Get session ID (try both formats - array or {data: array})
SESSIONS=$(curl -s "$BASE_URL/api/tasks/$TASK_ID/sessions")
SESSION_ID=$(echo "$SESSIONS" | jq -r 'if type == "array" then .[0].id else .data[0].id end // empty')

# If no session yet, try execute-async endpoint
if [ -z "$SESSION_ID" ] || [ "$SESSION_ID" == "null" ]; then
    echo "[$(date +%H:%M:%S.%3N)] No session yet, calling execute-async..."
    EXEC_RESULT=$(curl -s -X POST "$BASE_URL/api/tasks/$TASK_ID/execute-async" -H "Content-Type: application/json")
    SESSION_ID=$(echo "$EXEC_RESULT" | jq -r '.session_id // empty')
    echo "[$(date +%H:%M:%S.%3N)] Execute response: $(echo "$EXEC_RESULT" | jq -c '.')"
fi
echo "[$(date +%H:%M:%S.%3N)] Session ID: $SESSION_ID"

if [ -z "$SESSION_ID" ] || [ "$SESSION_ID" == "null" ]; then
    echo "ERROR: No session created"
    exit 1
fi

# Connect to SSE and monitor with timestamps
echo ""
echo "=== SSE Activity Stream ==="
echo ""

curl -s -N "$BASE_URL/api/sessions/$SESSION_ID/activity" | while IFS= read -r line; do
    TS=$(date +%H:%M:%S.%3N)

    if [[ "$line" =~ ^event:\ (.+) ]]; then
        EVENT_TYPE="${BASH_REMATCH[1]}"
    elif [[ "$line" =~ ^data:\ (.+) ]]; then
        DATA="${BASH_REMATCH[1]}"
        TYPE=$(echo "$DATA" | jq -r '.type // "?"')

        case "$TYPE" in
            step_start)
                echo -e "[$TS] \033[1;33mstep_start\033[0m"
                ;;
            reasoning)
                CONTENT=$(echo "$DATA" | jq -r '.content' | head -c 80)
                echo -e "[$TS] \033[0;34mreasoning\033[0m: ${CONTENT}..."
                ;;
            agent_message)
                CONTENT=$(echo "$DATA" | jq -r '.content' | head -c 80)
                IS_PARTIAL=$(echo "$DATA" | jq -r '.is_partial')
                echo -e "[$TS] \033[0;32magent_message\033[0m (partial=$IS_PARTIAL): ${CONTENT}..."
                ;;
            tool_call)
                TOOL=$(echo "$DATA" | jq -r '.tool_name')
                echo -e "[$TS] \033[0;36mtool_call\033[0m: $TOOL"
                ;;
            tool_result)
                TOOL=$(echo "$DATA" | jq -r '.tool_name')
                SUCCESS=$(echo "$DATA" | jq -r '.success')
                echo -e "[$TS] \033[0;36mtool_result\033[0m: $TOOL (success=$SUCCESS)"
                ;;
            finished)
                SUCCESS=$(echo "$DATA" | jq -r '.success')
                echo -e "[$TS] \033[0;31mfinished\033[0m (success=$SUCCESS)"
                echo ""
                echo "=== Stream Complete ==="
                break
                ;;
            *)
                echo "[$TS] $TYPE"
                ;;
        esac
    elif [[ "$line" == ": keep-alive" ]] || [[ "$line" == ":keep-alive" ]]; then
        echo -e "[$TS] \033[0;90mkeep-alive\033[0m"
    elif [[ "$line" == id:* ]]; then
        : # Skip ID lines
    elif [ -n "$line" ]; then
        echo "[$TS] $line"
    fi
done

echo ""
echo "[$(date +%H:%M:%S.%3N)] Test complete"
