#!/bin/bash

# End-to-end streaming test for opencode-os
# Tests the full flow: create task -> planning -> in_progress -> ai_review -> done
# Monitors SSE activity stream with timestamps

set -e

BASE_URL="${BASE_URL:-http://localhost:3001}"
TASK_TITLE="E2E Streaming Test $(date +%H:%M:%S)"
TASK_DESC="Testing real-time activity streaming across all phases"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

log() {
    echo -e "${CYAN}[$(date +%H:%M:%S.%3N)]${NC} $1"
}

log_event() {
    local type=$1
    local data=$2
    local color=$GREEN

    case "$type" in
        step_start) color=$YELLOW ;;
        reasoning) color=$BLUE ;;
        agent_message) color=$GREEN ;;
        tool_call) color=$CYAN ;;
        tool_result) color=$CYAN ;;
        finished) color=$RED ;;
    esac

    echo -e "${CYAN}[$(date +%H:%M:%S.%3N)]${NC} ${color}${type}${NC}: ${data:0:100}..."
}

# Check if server is running
log "Checking server health..."
if ! curl -s "$BASE_URL/health" > /dev/null; then
    echo -e "${RED}ERROR: Server not running at $BASE_URL${NC}"
    exit 1
fi
log "Server is healthy"

# Create task
log "Creating task: $TASK_TITLE"
TASK_RESPONSE=$(curl -s -X POST "$BASE_URL/api/tasks" \
    -H "Content-Type: application/json" \
    -d "{\"title\":\"$TASK_TITLE\",\"description\":\"$TASK_DESC\"}")

TASK_ID=$(echo "$TASK_RESPONSE" | jq -r '.id')
if [ -z "$TASK_ID" ] || [ "$TASK_ID" == "null" ]; then
    echo -e "${RED}ERROR: Failed to create task${NC}"
    echo "$TASK_RESPONSE"
    exit 1
fi
log "Created task: $TASK_ID"

# Function to monitor SSE stream for a session
monitor_session() {
    local session_id=$1
    local phase=$2
    local timeout=$3

    log "Connecting to SSE stream for session $session_id ($phase)..."

    # Create a temp file to capture output
    local tmpfile=$(mktemp)

    # Start SSE connection in background
    curl -s -N "$BASE_URL/api/sessions/$session_id/activity" > "$tmpfile" 2>&1 &
    local curl_pid=$!

    # Monitor the stream
    local start_time=$(date +%s)
    local last_size=0
    local finished=false

    while true; do
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))

        if [ $elapsed -ge $timeout ]; then
            log "Timeout after ${timeout}s"
            break
        fi

        # Check for new content
        local current_size=$(wc -c < "$tmpfile" 2>/dev/null || echo 0)
        if [ "$current_size" -gt "$last_size" ]; then
            # Extract new lines and process them
            tail -c +$((last_size + 1)) "$tmpfile" 2>/dev/null | while IFS= read -r line; do
                if [[ "$line" =~ ^event:\ (.+) ]]; then
                    event_type="${BASH_REMATCH[1]}"
                elif [[ "$line" =~ ^data:\ (.+) ]]; then
                    data="${BASH_REMATCH[1]}"
                    log_event "$event_type" "$data"

                    # Check if finished
                    if [ "$event_type" == "finished" ]; then
                        finished=true
                    fi
                fi
            done
            last_size=$current_size
        fi

        if [ "$finished" == "true" ]; then
            log "Session finished"
            break
        fi

        sleep 0.1
    done

    # Cleanup
    kill $curl_pid 2>/dev/null || true
    rm -f "$tmpfile"
}

# Function to execute a phase and monitor it
execute_phase() {
    local task_id=$1
    local target_status=$2
    local phase_name=$3
    local timeout=${4:-120}

    echo ""
    log "=========================================="
    log "Starting phase: $phase_name"
    log "=========================================="

    # Transition to target status (which triggers execution)
    log "Transitioning to $target_status..."
    local transition_response=$(curl -s -X POST "$BASE_URL/api/tasks/$task_id/transition" \
        -H "Content-Type: application/json" \
        -d "{\"status\":\"$target_status\"}")

    # Get session ID from the latest session
    sleep 0.5  # Wait for session to be created

    local sessions_response=$(curl -s "$BASE_URL/api/tasks/$task_id/sessions")
    local session_id=$(echo "$sessions_response" | jq -r '.data[-1].id // empty')

    if [ -z "$session_id" ]; then
        log "No session found, trying execute endpoint..."
        local exec_response=$(curl -s -X POST "$BASE_URL/api/tasks/$task_id/execute")
        session_id=$(echo "$exec_response" | jq -r '.session_id // empty')
    fi

    if [ -z "$session_id" ]; then
        echo -e "${RED}ERROR: Could not get session ID for $phase_name${NC}"
        return 1
    fi

    log "Session ID: $session_id"

    # Monitor the session
    monitor_session "$session_id" "$phase_name" "$timeout"

    # Check final task status
    local task_response=$(curl -s "$BASE_URL/api/tasks/$task_id")
    local current_status=$(echo "$task_response" | jq -r '.status')
    log "Task status after $phase_name: $current_status"
}

# Run the full E2E flow
echo ""
echo "=============================================="
echo "  E2E Streaming Test"
echo "  Task: $TASK_ID"
echo "  Started: $(date)"
echo "=============================================="

# Phase 1: Planning
execute_phase "$TASK_ID" "planning" "Planning" 180

# Wait a bit between phases
sleep 2

# Phase 2: In Progress (Implementation)
execute_phase "$TASK_ID" "in_progress" "Implementation" 300

# Wait a bit
sleep 2

# Phase 3: AI Review
execute_phase "$TASK_ID" "ai_review" "AI Review" 180

echo ""
echo "=============================================="
echo "  E2E Test Complete"
echo "  Task: $TASK_ID"
echo "  Finished: $(date)"
echo "=============================================="

# Final task status
FINAL_TASK=$(curl -s "$BASE_URL/api/tasks/$TASK_ID")
echo ""
log "Final task status: $(echo "$FINAL_TASK" | jq -r '.status')"
log "Task title: $(echo "$FINAL_TASK" | jq -r '.title')"
