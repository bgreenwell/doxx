# Homebrew Tap Automation Setup

## Quick Setup Checklist

### 1. Create Personal Access Token
1. Go to GitHub.com → Settings → Developer settings → Personal access tokens → Tokens (classic)
2. Click "Generate new token (classic)"
3. Name: `Homebrew Tap Automation`
4. Expiration: 90 days or longer
5. Scopes: Check ✅ **repo** (full repository access)
6. Click "Generate token"
7. **Copy the token** (starts with `ghp_`) - you won't see it again!

### 2. Add Token to Repository
1. Go to `bgreenwell/doxx` repository
2. Settings → Secrets and variables → Actions
3. Click "New repository secret"
4. Name: `HOMEBREW_TAP_TOKEN`
5. Secret: Paste the token from step 1
6. Click "Add secret"

### 3. Workflow is Ready
The workflow file `.github/workflows/update-homebrew.yml` is already created and will run automatically on each release.

## How It Works

When you publish a release:
1. Extracts version from tag (e.g., `v0.1.3` → `0.1.3`)
2. Downloads source tarball from GitHub
3. Calculates SHA256 hash
4. Updates `Formula/doxx.rb` in `homebrew-doxx` repo
5. Commits and pushes changes automatically

## Testing

After setting up the token, test with the next release:
1. Publish a new release (e.g., `v0.1.3`)
2. Go to Actions tab → "Update Homebrew Tap"
3. Verify workflow completes successfully
4. Check `bgreenwell/homebrew-doxx` for the commit

## Verification

After the workflow runs, verify the formula works:
```bash
brew tap bgreenwell/doxx
brew install doxx
doxx --version
```

## Troubleshooting

**Workflow doesn't run:**
- Ensure workflow file is committed to `main` branch
- Check that release is published (not draft)

**Permission denied:**
- Verify token has `repo` scope
- Check token hasn't expired
- Regenerate token if needed

**SHA256 mismatch:**
- GitHub may need a moment to generate the tarball
- Re-run the workflow manually from Actions tab

## Manual Trigger

If needed, you can manually trigger the workflow:
1. Go to Actions tab
2. Click "Update Homebrew Tap"
3. Click "Run workflow" dropdown
4. Click green "Run workflow" button

## Token Expiration

Set a calendar reminder before token expires to:
1. Generate a new token
2. Update `HOMEBREW_TAP_TOKEN` secret
3. Delete old token

## Status

- [x] Workflow file created
- [ ] Token added to repository secrets
- [ ] Tested with release
