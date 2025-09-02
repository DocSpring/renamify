#!/bin/bash
set -e

echo "Testing SIGTERM lock file cleanup..."
rm -f .renamify/renamify.lock

# Start the process in background
../target/debug/renamify plan test replace_test --include="**/*" > /dev/null 2>&1 &
PID=$!
echo "Started process $PID"

# Give it time to acquire lock
sleep 0.2

# Check if lock file was created
if [ -f .renamify/renamify.lock ]; then
    echo "Lock file created: $(cat .renamify/renamify.lock)"
else
    echo "No lock file found"
    exit 1
fi

# Send SIGTERM
echo "Sending SIGTERM to $PID"
kill -TERM $PID

# Wait for process to exit
sleep 1

# Check if lock file was cleaned up
if [ -f .renamify/renamify.lock ]; then
    echo "FAILURE: Lock file still exists: $(cat .renamify/renamify.lock)"
    rm -f .renamify/renamify.lock
    exit 1
else
    echo "SUCCESS: Lock file was cleaned up properly"
fi
