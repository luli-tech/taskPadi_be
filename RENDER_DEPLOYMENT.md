# Render Deployment Setup

This guide explains how to set up automatic deployment to Render via GitHub Actions.

## Prerequisites

1. A Render account (sign up at https://render.com)
2. Your GitHub repository connected to your Render account
3. A Web Service created on Render for this project

## Setup Instructions

### 1. Create a Web Service on Render

1. Go to https://dashboard.render.com
2. Click **"New +"** → **"Web Service"**
3. Connect your GitHub repository
4. Configure the service:
   - **Name**: `task-manager-api` (or your preferred name)
   - **Region**: Choose closest to your users
   - **Branch**: `main`
   - **Runtime**: `Rust`
   - **Build Command**: `cargo build --release`
   - **Start Command**: `./target/release/task-manager`
   - **Instance Type**: Choose based on your needs (Free tier available)

### 2. Configure Environment Variables on Render

Add the following environment variables in your Render service settings:

```
DATABASE_URL=<your-render-postgres-url>
JWT_SECRET=<your-production-jwt-secret>
JWT_EXPIRATION_HOURS=24
GOOGLE_CLIENT_ID=<your-google-client-id>
GOOGLE_CLIENT_SECRET=<your-google-client-secret>
GOOGLE_REDIRECT_URI=https://your-app.onrender.com/api/auth/google/callback
HOST=0.0.0.0
PORT=10000
RUST_LOG=info,task_manager=info
```

**Important Notes:**
- Render automatically provides `PORT` environment variable (usually 10000)
- Set `HOST=0.0.0.0` to allow external connections
- Update `GOOGLE_REDIRECT_URI` with your actual Render URL
- Generate a strong `JWT_SECRET` for production

### 3. Set Up PostgreSQL Database on Render

1. In Render Dashboard, click **"New +"** → **"PostgreSQL"**
2. Configure:
   - **Name**: `task-manager-db`
   - **Database**: `task_manager`
   - **User**: Auto-generated
   - **Region**: Same as your web service
   - **Plan**: Choose based on needs (Free tier available)
3. After creation, copy the **Internal Database URL**
4. Add it as `DATABASE_URL` environment variable in your web service

### 4. Get Render Deploy Hook URL

1. Go to your Web Service settings on Render
2. Navigate to **"Settings"** tab
3. Scroll down to **"Deploy Hook"** section
4. Click **"Create Deploy Hook"**
5. Copy the generated URL (it looks like: `https://api.render.com/deploy/srv-xxxxx?key=yyyyy`)

### 5. Add Deploy Hook to GitHub Secrets

1. Go to your GitHub repository
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **"New repository secret"**
4. Add:
   - **Name**: `RENDER_DEPLOY_HOOK_URL`
   - **Value**: Paste the Deploy Hook URL from Render
5. Click **"Add secret"**

## How It Works

The GitHub Actions workflow (`.github/workflows/ci.yml`) now includes two jobs:

### 1. Test Job
- Runs on every push and pull request
- Sets up PostgreSQL
- Runs database migrations
- Executes `cargo check`
- Runs all tests

### 2. Deploy Job
- **Only runs on `main` branch pushes** (not on PRs)
- **Requires test job to pass first**
- Triggers Render deployment via Deploy Hook
- Provides deployment status feedback

## Workflow Trigger Conditions

The deployment will **only** trigger when:
- ✅ Code is pushed to the `main` branch
- ✅ All tests pass successfully
- ❌ NOT on pull requests
- ❌ NOT on other branches

## Manual Deployment

You can also trigger manual deployments:

### Via Render Dashboard
1. Go to https://dashboard.render.com
2. Select your service
3. Click **"Manual Deploy"** → **"Deploy latest commit"**

### Via Deploy Hook (curl)
```bash
curl -X POST "https://api.render.com/deploy/srv-xxxxx?key=yyyyy"
```

## Monitoring Deployments

### View Deployment Logs
1. Go to Render Dashboard
2. Select your service
3. Click on **"Logs"** tab
4. View real-time deployment and runtime logs

### Check Deployment Status
1. Go to Render Dashboard
2. Select your service
3. View **"Events"** tab for deployment history

## Troubleshooting

### Deployment Fails

**Check Build Logs:**
- Go to Render Dashboard → Your Service → Logs
- Look for compilation errors or missing dependencies

**Common Issues:**
- Missing environment variables
- Database connection issues
- Incorrect build/start commands

### Database Migration Issues

Render automatically runs migrations on deployment if configured. To manually run migrations:

1. Go to Render Dashboard → Your Service
2. Click **"Shell"** tab
3. Run:
   ```bash
   sqlx migrate run
   ```

### Environment Variable Issues

Ensure all required environment variables are set:
- `DATABASE_URL` - PostgreSQL connection string
- `JWT_SECRET` - Strong secret key
- `GOOGLE_CLIENT_ID` - Google OAuth credentials
- `GOOGLE_CLIENT_SECRET` - Google OAuth credentials
- `GOOGLE_REDIRECT_URI` - Your Render app URL + callback path

## Production Checklist

Before deploying to production:

- [ ] Set strong `JWT_SECRET` (not the example value)
- [ ] Configure production `DATABASE_URL`
- [ ] Update `GOOGLE_REDIRECT_URI` to production URL
- [ ] Set `RUST_LOG=info` (not debug) for production
- [ ] Enable HTTPS (Render provides this automatically)
- [ ] Set up custom domain (optional)
- [ ] Configure CORS allowed origins for your frontend
- [ ] Test all endpoints in production
- [ ] Set up monitoring and alerts

## Automatic Deployments

Once configured, deployments happen automatically:

1. Developer pushes code to `main` branch
2. GitHub Actions runs tests
3. If tests pass, deployment is triggered
4. Render pulls latest code
5. Render builds the application
6. Render runs migrations (if configured)
7. Render starts the new version
8. Old version is replaced with zero downtime

## Rollback

If a deployment fails or has issues:

1. Go to Render Dashboard
2. Select your service
3. Click **"Rollback"** button
4. Select previous successful deployment
5. Confirm rollback

## Cost Considerations

### Free Tier Limits (as of 2024)
- **Web Service**: 750 hours/month (enough for 1 service running 24/7)
- **PostgreSQL**: 90 days free, then $7/month for 256MB
- **Automatic sleep after 15 minutes of inactivity** (free tier)
- **Cold start delay** when waking up (free tier)

### Paid Plans
- **Starter**: $7/month - No sleep, faster builds
- **Standard**: $25/month - More resources, better performance
- **Pro**: $85/month - Production-grade resources

## Support

- **Render Documentation**: https://render.com/docs
- **Render Community**: https://community.render.com
- **GitHub Actions Docs**: https://docs.github.com/en/actions

## Next Steps

After successful deployment:

1. Test your API at `https://your-app.onrender.com`
2. Check Swagger UI at `https://your-app.onrender.com/swagger-ui`
3. Update your frontend to use the production API URL
4. Set up monitoring and logging
5. Configure custom domain (optional)
