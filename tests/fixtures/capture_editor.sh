#!/bin/sh
# Test helper: capture the text the CLI pre-filled into the editor temp file.
# The destination must be provided via NBD_CAPTURE_PATH (absolute).
cp "$1" "${NBD_CAPTURE_PATH:?NBD_CAPTURE_PATH must be set}"