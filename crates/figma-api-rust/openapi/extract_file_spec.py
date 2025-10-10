"""
Extract file-only endpoints from the full Figma OpenAPI spec.

This script filters the full openapi.yaml to only include:
- /v1/files/* endpoints
- Components and Styles endpoints  
- Related schemas

Excluded paths:
- /v1/analytics/*
- /v1/payments/*
- /v1/activity_logs/*
- /v2/webhooks/*
- /v2/teams/*
- /v1/projects/*
- /v1/me
- /v1/dev_resources
- /v1/files/{file_key}/variables/*
- /v1/files/{file_key}/dev_resources/*
- /v1/files/{file_key}/comments/*
- /v1/files/{file_key}/versions/*
"""

from ruamel.yaml import YAML
from pathlib import Path
import re

yaml = YAML()
yaml.preserve_quotes = True
yaml.default_flow_style = False

# Load the full spec
with open("openapi.yaml", "r", encoding="utf-8") as f:
    full_spec = yaml.load(f)

# Paths to keep (patterns)
KEEP_PATH_PATTERNS = [
    r'^/v1/files/\{file_key\}$',
    r'^/v1/files/\{file_key\}/nodes$',
    r'^/v1/files/\{file_key\}/images$',
    r'^/v1/component_sets/\{key\}/components$',
    r'^/v1/components/\{key\}$',
    r'^/v1/styles/\{key\}$',
]

# Paths to explicitly exclude
EXCLUDE_PATH_PATTERNS = [
    r'^/v1/analytics/',
    r'^/v1/payments/',
    r'^/v1/activity_logs/',
    r'^/v2/webhooks/',
    r'^/v2/teams/',
    r'^/v1/projects/',
    r'^/v1/me$',
    r'^/v1/dev_resources',
    r'^/v1/files/\{file_key\}/variables/',
    r'^/v1/files/\{file_key\}/dev_resources/',
    r'^/v1/files/\{file_key\}/comments',
    r'^/v1/files/\{file_key\}/versions',
]

def should_keep_path(path):
    # Check if explicitly excluded
    for pattern in EXCLUDE_PATH_PATTERNS:
        if re.match(pattern, path):
            return False
    
    # Check if matches keep patterns
    for pattern in KEEP_PATH_PATTERNS:
        if re.match(pattern, path):
            return True
    
    return False

# Filter paths
filtered_paths = {}
for path, methods in full_spec.get("paths", {}).items():
    if should_keep_path(path):
        filtered_paths[path] = methods
        print(f"+ Keeping path: {path}")
    else:
        print(f"- Excluding path: {path}")

# Create the filtered spec
file_spec = {
    "openapi": full_spec["openapi"],
    "info": full_spec["info"],
    "servers": full_spec["servers"],
    "externalDocs": full_spec["externalDocs"],
    "tags": full_spec["tags"],
    "paths": filtered_paths,
    "components": full_spec["components"],  # Keep all components for now
}

# Save the filtered spec
with open("openapi.file.yaml", "w", encoding="utf-8") as f:
    yaml.dump(file_spec, f)
print(f"\nDone! Created openapi.file.yaml with {len(filtered_paths)} paths")

