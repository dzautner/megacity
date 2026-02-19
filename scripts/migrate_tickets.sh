#!/bin/bash
# Migrate ticket files to GitHub Issues and add to Project board
# Usage: ./scripts/migrate_tickets.sh [inbox|done|todo]

set -eo pipefail

REPO="dzautner/megacity"
PROJECT_ID="PVT_kwHOABlyX84BPl2K"
STATUS_FIELD_ID="PVTSSF_lAHOABlyX84BPl2Kzg99Mq4"
PRIORITY_FIELD_ID="PVTSSF_lAHOABlyX84BPl2Kzg99MxU"

get_status_id() {
  case "$1" in
    inbox) echo "5c37f8ac" ;;
    todo) echo "21a2764f" ;;
    in_progress) echo "7c056241" ;;
    testing) echo "19e0a706" ;;
    done) echo "45a70f0f" ;;
  esac
}

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

FOLDER="${1:-inbox}"
TICKET_DIR="docs/tickets/${FOLDER}"

if [ ! -d "$TICKET_DIR" ]; then
  echo "Directory $TICKET_DIR not found"
  exit 1
fi

# Map folder to status
case "$FOLDER" in
  inbox) STATUS="inbox" ;;
  todo) STATUS="todo" ;;
  doing) STATUS="in_progress" ;;
  done) STATUS="done" ;;
  *) STATUS="inbox" ;;
esac

STATUS_OPT_ID=$(get_status_id "$STATUS")

COUNT=0
TOTAL=$(ls "$TICKET_DIR"/*.md 2>/dev/null | wc -l | tr -d ' ')

for file in "$TICKET_DIR"/*.md; do
  [ -f "$file" ] || continue
  COUNT=$((COUNT + 1))

  FILENAME=$(basename "$file" .md)

  # Extract ticket ID (e.g., AUDIO-001)
  TICKET_ID=$(echo "$FILENAME" | grep -oE '^[A-Z]+-[0-9]+')

  # Extract category (e.g., AUDIO)
  CATEGORY=$(echo "$TICKET_ID" | sed 's/-[0-9]*//')

  # Handle INFR -> INFRA label mapping
  if [ "$CATEGORY" = "INFR" ]; then
    CATEGORY="INFRA"
  fi

  # Extract title from first H1 line
  TITLE=$(grep -m1 '^# ' "$file" | sed 's/^# //')
  if [ -z "$TITLE" ]; then
    TITLE="$FILENAME"
  fi

  # Extract priority
  PRIORITY=$(grep -m1 '^\*\*Priority:\*\*' "$file" | sed 's/\*\*Priority:\*\* //' | grep -oE 'T[0-5]' || echo "T3")

  # Build issue body from the file content
  BODY=$(cat "$file")

  # Build labels
  LABELS="${CATEGORY},${PRIORITY}"

  echo "[$COUNT/$TOTAL] Creating: $TITLE (${CATEGORY}, ${PRIORITY})"

  # Create the issue
  ISSUE_URL=$(gh issue create \
    --repo "$REPO" \
    --title "$TITLE" \
    --body "$BODY" \
    --label "$LABELS" \
    2>&1) || {
    echo "  FAILED to create issue: $ISSUE_URL"
    continue
  }

  # Extract issue number from URL
  ISSUE_NUM=$(echo "$ISSUE_URL" | grep -oE '[0-9]+$')

  # Get the issue node ID for project
  ISSUE_NODE_ID=$(gh api "repos/$REPO/issues/$ISSUE_NUM" --jq '.node_id' 2>/dev/null)

  if [ -z "$ISSUE_NODE_ID" ]; then
    echo "  Created issue #$ISSUE_NUM but couldn't get node ID"
    continue
  fi

  # Add to project
  ITEM_ID=$(gh api graphql -f query="
    mutation {
      addProjectV2ItemById(input: {
        projectId: \"$PROJECT_ID\"
        contentId: \"$ISSUE_NODE_ID\"
      }) {
        item { id }
      }
    }" --jq '.data.addProjectV2ItemById.item.id' 2>/dev/null)

  if [ -z "$ITEM_ID" ]; then
    echo "  Added issue #$ISSUE_NUM but couldn't add to project"
    continue
  fi

  # Set status
  gh api graphql -f query="
    mutation {
      updateProjectV2ItemFieldValue(input: {
        projectId: \"$PROJECT_ID\"
        itemId: \"$ITEM_ID\"
        fieldId: \"$STATUS_FIELD_ID\"
        value: {singleSelectOptionId: \"$STATUS_OPT_ID\"}
      }) { projectV2Item { id } }
    }" >/dev/null 2>&1

  # Set priority
  PRIO_ID=$(get_priority_id "$PRIORITY")
  if [ -n "$PRIO_ID" ]; then
    gh api graphql -f query="
      mutation {
        updateProjectV2ItemFieldValue(input: {
          projectId: \"$PROJECT_ID\"
          itemId: \"$ITEM_ID\"
          fieldId: \"$PRIORITY_FIELD_ID\"
          value: {singleSelectOptionId: \"$PRIO_ID\"}
        }) { projectV2Item { id } }
      }" >/dev/null 2>&1
  fi

  echo "  -> #$ISSUE_NUM [$STATUS] [$PRIORITY]"

  # Rate limit: GitHub allows ~30 requests/minute for mutations
  # Each issue needs ~4 API calls, so pause every 5 issues
  if [ $((COUNT % 5)) -eq 0 ]; then
    echo "  (pausing for rate limit...)"
    sleep 10
  fi
done

echo ""
echo "Done! Created $COUNT issues from $TICKET_DIR"
