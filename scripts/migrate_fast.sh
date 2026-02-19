#!/bin/bash
# Fast parallel migration - processes remaining inbox tickets
# Runs N workers in parallel, no unnecessary sleeps

set -eo pipefail

REPO="dzautner/megacity"
PROJECT_ID="PVT_kwHOABlyX84BPl2K"
STATUS_FIELD_ID="PVTSSF_lAHOABlyX84BPl2Kzg99Mq4"
PRIORITY_FIELD_ID="PVTSSF_lAHOABlyX84BPl2Kzg99MxU"
STATUS_OPT_ID="5c37f8ac"  # inbox

WORKERS=${1:-4}
START_AT=${2:-478}  # 1-indexed file number to start from

get_priority_id() {
  case "$1" in
    T0) echo "4a35384a" ;;
    T1) echo "70675d5e" ;;
    T2) echo "50aace90" ;;
    T3) echo "32c76242" ;;
    T4) echo "a800b2e2" ;;
    T5) echo "c87ad761" ;;
  esac
}

process_file() {
  local file="$1"
  local num="$2"
  local total="$3"

  FILENAME=$(basename "$file" .md)
  TICKET_ID=$(echo "$FILENAME" | grep -oE '^[A-Z]+-[0-9]+')
  CATEGORY=$(echo "$TICKET_ID" | sed 's/-[0-9]*//')
  [ "$CATEGORY" = "INFR" ] && CATEGORY="INFRA"

  TITLE=$(grep -m1 '^# ' "$file" | sed 's/^# //')
  [ -z "$TITLE" ] && TITLE="$FILENAME"

  PRIORITY=$(grep -m1 'Priority:' "$file" | grep -oE 'T[0-5]' || echo "T3")
  BODY=$(cat "$file")
  LABELS="${CATEGORY},${PRIORITY}"

  # Create issue
  ISSUE_URL=$(gh issue create --repo "$REPO" --title "$TITLE" --body "$BODY" --label "$LABELS" 2>&1) || {
    echo "[$num/$total] FAIL: $TITLE"
    return 1
  }

  ISSUE_NUM=$(echo "$ISSUE_URL" | grep -oE '[0-9]+$')
  ISSUE_NODE_ID=$(gh api "repos/$REPO/issues/$ISSUE_NUM" --jq '.node_id' 2>/dev/null)

  [ -z "$ISSUE_NODE_ID" ] && { echo "[$num/$total] #$ISSUE_NUM (no node ID)"; return 0; }

  # Add to project + set fields in one shot
  ITEM_ID=$(gh api graphql -f query="
    mutation {
      addProjectV2ItemById(input: {
        projectId: \"$PROJECT_ID\"
        contentId: \"$ISSUE_NODE_ID\"
      }) { item { id } }
    }" --jq '.data.addProjectV2ItemById.item.id' 2>/dev/null)

  [ -n "$ITEM_ID" ] && {
    gh api graphql -f query="mutation { updateProjectV2ItemFieldValue(input: { projectId: \"$PROJECT_ID\" itemId: \"$ITEM_ID\" fieldId: \"$STATUS_FIELD_ID\" value: {singleSelectOptionId: \"$STATUS_OPT_ID\"} }) { projectV2Item { id } } }" >/dev/null 2>&1
    PRIO_ID=$(get_priority_id "$PRIORITY")
    [ -n "$PRIO_ID" ] && gh api graphql -f query="mutation { updateProjectV2ItemFieldValue(input: { projectId: \"$PROJECT_ID\" itemId: \"$ITEM_ID\" fieldId: \"$PRIORITY_FIELD_ID\" value: {singleSelectOptionId: \"$PRIO_ID\"} }) { projectV2Item { id } } }" >/dev/null 2>&1
  }

  echo "[$num/$total] #$ISSUE_NUM $TICKET_ID [$PRIORITY]"
}

export -f process_file get_priority_id
export REPO PROJECT_ID STATUS_FIELD_ID PRIORITY_FIELD_ID STATUS_OPT_ID

# Get remaining files
FILES=(docs/tickets/inbox/*.md)
TOTAL=${#FILES[@]}
REMAINING=$((TOTAL - START_AT + 1))

echo "Starting from file $START_AT/$TOTAL ($REMAINING remaining, $WORKERS workers)"

# Process with GNU parallel-style using background jobs
RUNNING=0
for i in $(seq $START_AT $TOTAL); do
  idx=$((i - 1))
  file="${FILES[$idx]}"

  process_file "$file" "$i" "$TOTAL" &
  RUNNING=$((RUNNING + 1))

  if [ $RUNNING -ge $WORKERS ]; then
    wait -n 2>/dev/null || wait
    RUNNING=$((RUNNING - 1))
  fi
done

wait
echo "Done! Migrated remaining tickets."
