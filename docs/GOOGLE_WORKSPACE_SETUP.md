# Google Workspace OAuth 2.0 Setup Guide

This guide explains how to configure authentication for the Google Workspace tool in ZeroClaw.

## Two Authentication Methods

| Method | Use When | Variables |
|--------|----------|-----------|
| OAuth 2.0 | Personal Gmail (@gmail.com) | `GOOGLE_REFRESH_TOKEN`, `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET` |
| Service Account | Google Workspace/Organization | `GOOGLE_SERVICE_ACCOUNT_PATH` or `GOOGLE_SERVICE_ACCOUNT_JSON` |

---

## Method 1: OAuth 2.0 (Personal Gmail)

### Step 1: Create OAuth Credentials

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project (or use existing)
3. Go to **APIs & Services > Library**
4. Enable these APIs:
   - Gmail API
   - Google Drive API
   - Google Calendar API
   - Google Docs API
   - Google Sheets API
   - Google Slides API
   - Google Chat API

5. Go to **APIs & Services > OAuth consent screen**
6. Configure:
   - User Type: **External**
   - Email: your personal Gmail
   - Scopes: Add the scopes from below

**Scopes to add:**
```
https://www.googleapis.com/auth/gmail.readonly
https://www.googleapis.com/auth/gmail.send
https://www.googleapis.com/auth/drive
https://www.googleapis.com/auth/drive.file
https://www.googleapis.com/auth/calendar
https://www.googleapis.com/auth/documents
https://www.googleapis.com/auth/spreadsheets
https://www.googleapis.com/auth/presentations
https://www.googleapis.com/auth/chat.metadata
```

7. Go to **APIs & Services > Credentials**
8. Click **Create Credentials > OAuth client ID**
9. Application type: **Desktop app**
10. Download the JSON file

### Step 2: Get Refresh Token

Run the script:

```bash
# Install dependencies
pip install requests

# Run the script
python scripts/get_google_refresh_token.py --credentials path/to/client_secret.json
```

The script will:
1. Open your browser for authorization
2. Exchange the code for a refresh token
3. Save credentials to `google_credentials.json`

### Step 3: Configure Environment Variables

```bash
# In your .env file or Render environment

GOOGLE_REFRESH_TOKEN=1//xxx...
GOOGLE_CLIENT_ID=xxx.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=xxx
```

---

## Method 2: Service Account (Google Workspace)

### Step 1: Create Service Account

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Go to **IAM & Admin > Service Accounts**
3. Click **+ Create Service Account**
4. Fill in details and click **Create**
5. Grant role: **Project > Owner** (or specific roles)
6. Click **Done**

### Step 2: Download Key

1. Find your service account in the list
2. Click the 3 dots > **Manage keys**
3. Click **+ Add key > Create new key**
4. Select **JSON**
5. Click **Create** - file will download

### Step 3: Configure Environment Variables

```bash
# Option A: Path to file
GOOGLE_SERVICE_ACCOUNT_PATH=/path/to/service-account.json

# Option B: JSON content (for Render secrets)
GOOGLE_SERVICE_ACCOUNT_JSON='{"type":"service_account",...}'
```

### Step 4: Grant Access

For Gmail/Drive/Calendar access:
- Share the resources with the service account email
- Email format: `name@project.iam.gserviceaccount.com`

---

## Quick Reference

### Environment Variables

```bash
# OAuth 2.0 (personal Gmail)
GOOGLE_REFRESH_TOKEN=your_refresh_token
GOOGLE_CLIENT_ID=your_client_id
GOOGLE_CLIENT_SECRET=your_client_secret

# Service Account (workspace)
GOOGLE_SERVICE_ACCOUNT_PATH=/path/to/credentials.json
# OR
GOOGLE_SERVICE_ACCOUNT_JSON='{"type":"service_account",...}'
```

### Tool Usage Examples

```json
{
  "service": "gmail",
  "action": "list",
  "params": { "max_results": 10 }
}
```

```json
{
  "service": "gmail",
  "action": "send",
  "params": {
    "to": "user@example.com",
    "subject": "Hello",
    "body": "World"
  }
}
```

```json
{
  "service": "sheets",
  "action": "create",
  "params": { "title": "My Spreadsheet" }
}
```

---

## Troubleshooting

### "invalid_grant" Error
- Refresh token may be expired or revoked
- Re-run the authorization script to get a new refresh token

### "access_denied" Error
- OAuth consent not fully configured
- Go to OAuth consent screen and verify status shows "Published"

### API Errors
- Make sure all required APIs are enabled in Google Cloud Console
- Verify service account has access to the resources (for Workspace)

### Token Expiry
- Access tokens expire after 1 hour
- Refresh tokens don't expire (unless revoked)
- The tool automatically refreshes when needed
