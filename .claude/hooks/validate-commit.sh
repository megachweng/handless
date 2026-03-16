#!/bin/bash

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command')

# Only intercept git commit commands
if ! echo "$COMMAND" | grep -q '^git commit'; then
  exit 0
fi

# Run lint check
if ! bun run lint 2>&1; then
  echo '{"decision":"block","reason":"Lint check failed. Fix lint errors before committing."}'
  exit 2
fi

# Run translation check
if ! bun run check:translations 2>&1; then
  echo '{"decision":"block","reason":"Translation check failed. All locale files must have matching keys."}'
  exit 2
fi

exit 0
