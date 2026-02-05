# uqgrd

A CLI tool and background daemon for monitoring UQAM student grades. It interfaces directly with the `monportail.uqam.ca` API to fetch transcripts, parse course details, and send SMTP notifications when new grades or percentage updates are detected.

## Features

- **CLI Access:** View full academic history and detailed course grades directly in the terminal.
- **Current Semester Detection:** Automatically identifies the active semester based on the current date.
- **Daemon Mode:** Runs in the background to poll for changes at a configurable interval.
- **Email Alerts:** Sends SMTP notifications immediately upon detecting a grade change.
- **Containerized:** Includes scripts for deployment via Podman or Docker.

## Requirements

- Rust (Latest Stable)
- Podman or Docker (for deployment)
- OpenSSL (libssl-dev)

## Build

```bash
cargo build --release
```

## Usage

### 1. Setup Credentials

Store your UQAM `Code permanent` and password locally.

```bash
./target/release/uqgrd credentials
```

### 2. View Grades

Interactively select a semester to view:

```bash
./target/release/uqgrd grades
```

View the current semester automatically:

```bash
./target/release/uqgrd grades --current
```

### 3. Daemon Mode

Starts the monitoring loop. This requires environment variables for SMTP configuration (see Deployment).

```bash
./target/release/uqgrd start
```

## Deployment

A `deploy.sh` script is provided to build the container, configure credentials, and start the daemon using Podman (or Docker).

### Steps

1. **Run the deployment script:**

```bash
./deploy.sh
```

On the first run, it will generate a `.env` file and exit. 2. **Configure Environment:**
Edit the `.env` file with your SMTP details:

```ini
CHECK_INTERVAL=60
SMTP_SERVER=smtp.gmail.com
SMTP_USERNAME=your.email@gmail.com
SMTP_PASSWORD=your-app-password
```

3. **Start the Service:**
   Run the script again to build and start the container.

```bash
./deploy.sh
```

### Logs

Monitor the background process:

```bash
podman logs -f uqgrd
```

### Cleanup

Stop the service and remove artifacts:

```bash
./cleanup.sh
```

## Configuration

### File Locations

- **Credentials:** `$HOME/.config/uqgrd/config.json`
- **Grade State:** `$HOME/.config/uqgrd/grades_state.json` (Used for diffing)

### Environment Variables (Daemon)

| Variable         | Description                  | Default        |
| ---------------- | ---------------------------- | -------------- |
| `CHECK_INTERVAL` | Polling frequency in minutes | 60             |
| `SMTP_SERVER`    | SMTP Hostname                | smtp.gmail.com |
| `SMTP_USERNAME`  | SMTP User                    | N/A            |
| `SMTP_PASSWORD`  | SMTP Password/App Password   | N/A            |
