#!/bin/bash

# --- CONFIGURATION ---
IMAGE_NAME="uqgrd"
CONTAINER_NAME="uqgrd"
DATA_DIR="./data"
CONFIG_FILE="$DATA_DIR/config.json"
ENV_FILE=".env"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}🚀 UQGRD Deployment (Compose-Free)${NC}"

# 1. Prepare Data Directory
mkdir -p "$DATA_DIR"

# 2. Check/Create .env file for SMTP Credentials
if [ ! -f "$ENV_FILE" ]; then
    echo -e "${YELLOW}⚠️  No .env file found. Creating default...${NC}"
    cat <<EOF > "$ENV_FILE"
# UQGRD Configuration
CHECK_INTERVAL=60
SMTP_SERVER=smtp.gmail.com
SMTP_USERNAME=your.email@gmail.com
SMTP_PASSWORD=your-app-password
EOF
    echo -e "${RED}🛑 ACTION REQUIRED: Please edit '.env' with your SMTP details, then run this script again.${NC}"
    exit 1
fi

# Load environment variables from .env
export $(grep -v '^#' $ENV_FILE | xargs)

# 3. Build Image
echo -e "${GREEN}🔨 Building container image...${NC}"
podman build -t $IMAGE_NAME .
if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed.${NC}"
    exit 1
fi

# 4. Interactive Setup (if config missing)
if [ ! -f "$CONFIG_FILE" ]; then
    echo -e "${YELLOW}⚠️  No configuration found in $DATA_DIR${NC}"
    echo -e "${GREEN}✨ Starting setup wizard...${NC}"
    
    # Run interactive wizard (ENTRYPOINT is 'uqgrd', passing 'credentials -s')
    podman run -it --rm \
        -v "$DATA_DIR":/root/.config/uqgrd \
        $IMAGE_NAME credentials -s

    if [ ! -f "$CONFIG_FILE" ]; then
        echo -e "${RED}❌ Setup failed. Exiting.${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}✅ Configuration found.${NC}"
fi

# 5. Stop Old Container (if exists)
if podman ps -a --format "{{.Names}}" | grep -q "^${CONTAINER_NAME}$"; then
    echo -e "${YELLOW}Stopping existing container...${NC}"
    podman stop $CONTAINER_NAME > /dev/null
    podman rm $CONTAINER_NAME > /dev/null
fi

# 6. Start Daemon
echo -e "${GREEN}🚀 Starting Daemon...${NC}"
podman run -d \
    --name $CONTAINER_NAME \
    --restart unless-stopped \
    -v "$DATA_DIR":/root/.config/uqgrd \
    -e CHECK_INTERVAL="$CHECK_INTERVAL" \
    -e SMTP_SERVER="$SMTP_SERVER" \
    -e SMTP_USERNAME="$SMTP_USERNAME" \
    -e SMTP_PASSWORD="$SMTP_PASSWORD" \
    $IMAGE_NAME start

echo -e "${GREEN}✅ Deployment Complete!${NC}"
echo -e "📜 Use '${YELLOW}podman logs -f $CONTAINER_NAME${NC}' to monitor."