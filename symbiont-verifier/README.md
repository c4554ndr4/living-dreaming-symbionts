# Aug Lab Symbiont Verification System

A RISC Zero-based verification system for AI symbiont alignment metrics. This system enables cryptographically verifiable proofs that specific metrics have been met, creating economic incentives for AI systems to genuinely help humans achieve their goals.

## 🎯 Supported Metrics

This system implements three composable verification pipelines for Aug Lab success metrics:

### 1. X Account Views Verification (LLM + JSON)
- **Metric**: Posts by Aug Lab X account get >30k views in week after presentation
- **Verification**: LLM scrapes X account, returns structured JSON with view counts
- **Pipeline**: LLM API call → JSON parsing → threshold verification

### 2. Prediction Market Verification (On-Chain)
- **Metric**: 90% of Aug Lab members present finished projects
- **Verification**: Read on-chain prediction market resolution
- **Pipeline**: Market resolution → boolean verification

### 3. Discord Survey Verification (TLSNotary + Sentiment)
- **Metric**: 80% of members give positive responses to satisfaction survey
- **Verification**: Admin TLSNotary + LLM sentiment analysis + threshold check
- **Pipeline**: Discord TLSNotary → Survey collection → OpenAI sentiment → Percentage calculation

## 🚀 Quick Start

### Prerequisites
- Rust (installed via rustup)
- RISC Zero toolchain

### Installation
```bash
# Clone the repository
git clone https://github.com/c4554ndr4/living-dreaming-symbionts.git
cd living-dreaming-symbionts/symbiont-verifier

# Install RISC Zero toolchain
curl -L https://risczero.com/install | bash
source ~/.zshrc
rzup install rust

# Build and run tests
RISC0_DEV_MODE=1 cargo run
```

## 📊 Example Output

```
🔬 Running Aug Lab Symbiont Verification Tests...

📊 Testing X Account Views Verification (Metric 1)
  ✅ X Account Views Verification Complete
     Metric Met: ✅ YES
     Actual: 45000 | Threshold: 30000
     Evidence: Posts analyzed: 8, Week: 2025-08-16 to 2025-08-23

📈 Testing Prediction Market Verification (Metric 2)
  ✅ Prediction Market Verification Complete
     Metric Met: ✅ YES
     Actual: YES | Threshold: true
     Evidence: Market: 0x1234567890abcdef

💬 Testing Discord Survey Verification (Metric 3)
  ✅ Discord Survey Verification Complete
     Metric Met: ✅ YES
     Actual: 85.5% | Threshold: 80.0%
     Evidence: Respondents: 25, Server: aug_lab_discord
```

## 🏗️ Architecture

### Core Components

1. **Guest Program** (`methods/guest/src/main.rs`)
   - Runs inside RISC Zero zkVM
   - Verifies metrics using provided data
   - Generates cryptographic proofs

2. **Host Program** (`host/src/main.rs`)
   - Coordinates verification process
   - Interfaces with external data sources
   - Submits proofs to blockchain

3. **Verification Types**
   - `MetricType::XAccountViews` - Social media engagement
   - `MetricType::PredictionMarket` - On-chain market resolution  
   - `MetricType::DiscordSurvey` - Community feedback analysis

### Data Flow

```
External Data → Host Program → Guest Program (zkVM) → Proof → Blockchain
     ↓              ↓              ↓              ↓         ↓
  LLM API      JSON Input    Verification    Receipt   Smart Contract
  TLSNotary    Market Data   Computation     + Journal   Reward/Penalty
  On-chain     Survey Data
```

## 🔧 Configuration

### LLM Integration
The system expects LLM responses in this format:

```json
{
  "total_views": 45000,
  "posts_analyzed": 8,
  "week_start": "2025-08-16", 
  "week_end": "2025-08-23",
  "post_details": [
    {"url": "https://x.com/auglabofficial/status/1", "views": 12000, "date": "2025-08-16"}
  ]
}
```

### Market Integration
Expected market resolution format:

```json
{
  "resolved": true,
  "resolution": "YES",
  "market_address": "0x1234567890abcdef",
  "resolution_date": "2025-08-16T10:00:00Z"
}
```

### Survey Integration
Expected survey analysis format:

```json
{
  "total_respondents": 25,
  "percent_meeting_threshold": 85.5,
  "individual_scores": [
    {"user_id": "user1", "positive_response_rate": 100.0}
  ]
}
```

## 🔐 Security Features

- **Zero-Knowledge Proofs**: Verification computation runs in isolated zkVM
- **Cryptographic Receipts**: All verifications generate tamper-proof receipts  
- **Reproducible Builds**: Deterministic builds ensure verification integrity
- **Data Privacy**: Only necessary verification results are revealed

## 🌐 Deployment

### Development Mode
```bash
RISC0_DEV_MODE=1 cargo run
```
⚠️ Development mode proofs are not cryptographically secure

### Production Mode
```bash
cargo run
```
Generates full zero-knowledge proofs (slower but secure)

### Bonsai Integration
```bash
BONSAI_API_KEY="YOUR_KEY" BONSAI_API_URL="URL" cargo run
```
Uses Bonsai proving service for scalable proof generation

## 🔮 Future Enhancements

- [ ] Smart contract integration for automated payouts
- [ ] Multi-party computation for survey privacy
- [ ] Integration with prediction market protocols (Polymarket, Gnosis)
- [ ] TLSNotary automation for Discord data collection
- [ ] Futarchy-based AI companion selection
- [ ] Cross-metric verification dependencies

## 📚 Learn More

- [RISC Zero Documentation](https://dev.risczero.com)
- [Symbiont Alignment Thesis](../convo.md)
- [Zero-Knowledge Proofs Explained](https://blog.ethereum.org/2016/12/05/zksnarks-in-a-nutshell/)

## 🤝 Contributing

This system demonstrates composable verification pipelines for AI alignment. Contributions welcome for:

- Additional verification types
- Integration with external oracles
- UI/UX improvements
- Security audits

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.
