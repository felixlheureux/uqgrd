#!/bin/bash

# --- CONFIGURATION ---
CONTAINER_NAME="uqgrd"
IMAGE_NAME="uqgrd"
DATA_DIR="./data"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${RED}🧹 UQGRD Cleanup Script${NC}"

# 1. Stop and Remove Container
if podman ps -a --format "{{.Names}}" | grep -q "^${CONTAINER_NAME}$"; then
    echo -e "${YELLOW}Stopping and removing container...${NC}"
    # Try compose down first if it was started that way
    if command -v podman-compose &> /dev/null; then
        podman-compose down
    else
        podman stop $CONTAINER_NAME
        podman rm $CONTAINER_NAME
    fi
    echo -e "${GREEN}✅ Container removed.${NC}"
else
    echo "No running container found."
fi

# 2. Remove Image
if podman images --format "{{.Repository}}" | grep -q "^${IMAGE_NAME}$"; then
    echo -e "${YELLOW}Removing image '$IMAGE_NAME'...${NC}"
    podman rmi $IMAGE_NAME
    echo -e "${GREEN}✅ Image removed.${NC}"
else
    echo "No image found."
fi

# 3. Clean Data/Credentials
if [ -d "$DATA_DIR" ]; then
    echo -e "${RED}⚠️  WARNING: This will delete your saved credentials and grade history!${NC}"
    read -p "❓ Delete data directory ($DATA_DIR)? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}Removing $DATA_DIR...${NC}"
        rm -rf "$DATA_DIR"
        echo -e "${GREEN}✅ Data cleaned.${NC}"
    else
        echo -e "${GREEN}Data preserved.${NC}"
    fi
fi

echo -e "${GREEN}✨ Cleanup complete!${NC}"