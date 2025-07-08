# 🔑 THE OVERMIND PROTOCOL - API Keys Setup Guide

This guide will walk you through obtaining and configuring all required API keys for THE OVERMIND PROTOCOL production deployment.

## 📋 **REQUIRED API KEYS CHECKLIST**

- [ ] **Helius API Key** (Critical for Solana data)
- [ ] **Jito v2 API Key** (Critical for MEV execution)
- [ ] **DeepSeek V2 API Key** (AI reasoning)
- [ ] **Jina AI API Key** (Embeddings & reranking)
- [ ] **TensorZero API Key** (AI optimization - Optional)

---

## 🌐 **1. HELIUS API KEY**

**Purpose:** High-performance Solana RPC and WebSocket streaming
**Criticality:** REQUIRED for production
**Cost:** Paid service with free tier

### Setup Steps:

1. **Visit Helius Dashboard**
   ```
   https://dashboard.helius.dev/
   ```

2. **Create Account & Project**
   - Sign up with email
   - Create new project: "THE OVERMIND PROTOCOL"
   - Select "Mainnet" network

3. **Get API Key**
   - Navigate to "API Keys" section
   - Copy your API key (starts with your project name)
   - Example format: `your-project-name-abc123def456`

4. **Configure Rate Limits**
   - Recommended: Professional plan (1M+ requests/day)
   - Enable WebSocket streaming
   - Enable enhanced APIs

5. **Add to Environment**
   ```bash
   HELIUS_API_KEY=your-actual-helius-api-key-here
   HELIUS_RPC_URL=https://mainnet.helius-rpc.com/?api-key=your-actual-helius-api-key-here
   HELIUS_WEBSOCKET_URL=wss://mainnet.helius-rpc.com/?api-key=your-actual-helius-api-key-here
   ```

---

## ⚡ **2. JITO V2 API KEY**

**Purpose:** MEV bundle submission and block space auction
**Criticality:** REQUIRED for MEV strategies
**Cost:** Performance-based fees

### Setup Steps:

1. **Visit Jito Foundation**
   ```
   https://www.jito.wtf/
   ```

2. **Apply for API Access**
   - Fill out application form
   - Describe your MEV strategy
   - Provide technical details about THE OVERMIND PROTOCOL

3. **Get Approved**
   - Wait for approval (usually 1-3 business days)
   - Receive API key via email

4. **Configure Bundle Settings**
   - Set up tip account: `96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5`
   - Configure validator preferences
   - Set priority fee strategies

5. **Add to Environment**
   ```bash
   JITO_API_KEY=your-actual-jito-api-key-here
   JITO_BUNDLE_URL=https://mainnet.block-engine.jito.wtf/api/v1/bundles
   JITO_TIP_ACCOUNT=96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5
   ```

---

## 🧠 **3. DEEPSEEK V2 API KEY**

**Purpose:** Advanced AI reasoning for trading decisions
**Criticality:** REQUIRED for AI features
**Cost:** Pay-per-token usage

### Setup Steps:

1. **Visit DeepSeek Platform**
   ```
   https://platform.deepseek.com/
   ```

2. **Create Account**
   - Sign up with email or GitHub
   - Verify your account

3. **Get API Key**
   - Navigate to "API Keys" section
   - Create new API key: "THE OVERMIND PROTOCOL"
   - Copy the key (starts with `sk-`)

4. **Add Credits**
   - Add initial credits ($20-50 recommended)
   - Monitor usage in dashboard

5. **Add to Environment**
   ```bash
   DEEPSEEK_API_KEY=sk-your-actual-deepseek-api-key-here
   DEEPSEEK_BASE_URL=https://api.deepseek.com
   ```

---

## 🔍 **4. JINA AI API KEY**

**Purpose:** Enhanced embeddings and reranking for AI analysis
**Criticality:** REQUIRED for advanced AI features
**Cost:** Free tier available, paid for higher usage

### Setup Steps:

1. **Visit Jina AI Dashboard**
   ```
   https://jina.ai/api-dashboard/
   ```

2. **Create Account**
   - Sign up with email
   - Verify your account

3. **Get API Key**
   - Navigate to API section
   - Create new API key
   - Copy the key (format: `jina_...`)

4. **Configure Services**
   - Enable Embeddings API
   - Enable Reranker API
   - Set usage limits

5. **Add to Environment**
   ```bash
   JINA_API_KEY=jina_your-actual-jina-api-key-here
   JINA_BASE_URL=https://api.jina.ai
   ```

---

## 🎯 **5. TENSORZERO API KEY (OPTIONAL)**

**Purpose:** AI model optimization and performance tuning
**Criticality:** OPTIONAL (enhances AI performance)
**Cost:** Varies by usage

### Setup Steps:

1. **Visit TensorZero**
   ```
   https://tensorzero.com/
   ```

2. **Request Access**
   - Apply for beta access
   - Describe your use case

3. **Get API Key**
   - Receive invitation email
   - Create account and get API key

4. **Add to Environment**
   ```bash
   TENSORZERO_API_KEY=your-actual-tensorzero-api-key-here
   TENSORZERO_BASE_URL=https://api.tensorzero.com
   ```

---

## 🔐 **SECURITY BEST PRACTICES**

### 1. **Environment File Security**
```bash
# Set proper permissions
chmod 600 .env

# Never commit to version control
echo ".env" >> .gitignore
```

### 2. **API Key Rotation**
```bash
# Rotate keys monthly
# Keep backup keys ready
# Monitor usage for anomalies
```

### 3. **Rate Limit Monitoring**
```bash
# Monitor API usage
# Set up alerts for high usage
# Have backup keys ready
```

### 4. **Access Control**
```bash
# Limit API key permissions
# Use separate keys for different environments
# Monitor access logs
```

---

## 🧪 **TESTING API KEYS**

Before production deployment, test all API keys:

### 1. **Test Helius Connection**
```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
  https://mainnet.helius-rpc.com/?api-key=YOUR_KEY
```

### 2. **Test Jito Access**
```bash
curl -X GET \
  -H "Authorization: Bearer YOUR_JITO_KEY" \
  https://mainnet.block-engine.jito.wtf/api/v1/validators
```

### 3. **Test DeepSeek API**
```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_DEEPSEEK_KEY" \
  -d '{"model":"deepseek-chat","messages":[{"role":"user","content":"test"}]}' \
  https://api.deepseek.com/v1/chat/completions
```

### 4. **Test Jina AI**
```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JINA_KEY" \
  -d '{"input":["test text"],"model":"jina-embeddings-v2-base-en"}' \
  https://api.jina.ai/v1/embeddings
```

---

## 📊 **COST ESTIMATION**

### Monthly API Costs (Estimated):

| Service | Free Tier | Recommended Plan | Monthly Cost |
|---------|-----------|------------------|--------------|
| **Helius** | 100K requests | Professional | $99/month |
| **Jito v2** | N/A | Performance-based | $200-500/month |
| **DeepSeek** | $5 credit | Pay-per-use | $50-100/month |
| **Jina AI** | 1M tokens | Pro | $20/month |
| **TensorZero** | Beta | Custom | $50/month |
| **TOTAL** | - | - | **$419-669/month** |

---

## ⚠️ **IMPORTANT NOTES**

1. **Never share API keys** in public repositories or communications
2. **Monitor usage** to avoid unexpected charges
3. **Set up billing alerts** for all services
4. **Keep backup keys** ready for failover
5. **Rotate keys regularly** for security
6. **Test in development** before production use

---

## 🆘 **TROUBLESHOOTING**

### Common Issues:

1. **"Invalid API Key" Error**
   - Check key format and spelling
   - Verify key is active in dashboard
   - Check rate limits

2. **"Rate Limit Exceeded"**
   - Upgrade to higher tier
   - Implement request throttling
   - Use multiple keys for load balancing

3. **"Insufficient Credits"**
   - Add more credits to account
   - Set up auto-recharge
   - Monitor usage patterns

4. **"Service Unavailable"**
   - Check service status pages
   - Implement fallback mechanisms
   - Use backup endpoints

---

## 📞 **SUPPORT CONTACTS**

- **Helius Support:** support@helius.dev
- **Jito Support:** support@jito.wtf
- **DeepSeek Support:** support@deepseek.com
- **Jina AI Support:** support@jina.ai

---

**🚀 Once all API keys are configured, you're ready to deploy THE OVERMIND PROTOCOL to production!**
