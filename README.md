# LLM Council

![llmcouncil](header.jpg)

> **🚀 Now with Rust Backend!** This project has been migrated from Python/FastAPI to Rust/Axum for better performance and native desktop integration via Tauri. The Python backend is archived in `backend-python-backup/` for reference.

The idea of this repo is that instead of asking a question to your favorite LLM provider (e.g. OpenAI GPT 5.1, Google Gemini 3.0 Pro, Anthropic Claude Sonnet 4.5, xAI Grok 4, eg.c), you can group them into your "LLM Council". This is a desktop application (or web app) that essentially looks like ChatGPT except it uses OpenRouter to send your query to multiple LLMs, it then asks them to review and rank each other's work, and finally a Chairman LLM produces the final response.

In a bit more detail, here is what happens when you submit a query:

1. **Stage 1: First opinions**. The user query is given to all LLMs individually, and the responses are collected. The individual responses are shown in a "tab view", so that the user can inspect them all one by one.
2. **Stage 2: Review**. Each individual LLM is given the responses of the other LLMs. Under the hood, the LLM identities are anonymized so that the LLM can't play favorites when judging their outputs. The LLM is asked to rank them in accuracy and insight.
3. **Stage 3: Final response**. The designated Chairman of the LLM Council takes all of the model's responses and compiles them into a single final answer that is presented to the user.

## Vibe Code Alert

This project was 99% vibe coded as a fun Saturday hack because I wanted to explore and evaluate a number of LLMs side by side in the process of [reading books together with LLMs](https://x.com/karpathy/status/1990577951671509438). It's nice and useful to see multiple responses side by side, and also the cross-opinions of all LLMs on each other's outputs. I'm not going to support it in any way, it's provided here as is for other people's inspiration and I don't intend to improve it. Code is ephemeral now and libraries are over, ask your LLM to change it in whatever way you like.

## Setup

### 1. Install Dependencies

**Rust Backend (Required):**
```bash
# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Tauri CLI globally
npm install -g @tauri-apps/cli

# Install root dependencies
npm install

# Install frontend dependencies
cd frontend
npm install
cd ..
```

### 2. Configure API Key

Create a `.env` file in the project root:

```bash
OPENROUTER_API_KEY=sk-or-v1-...
```

Get your API key at [openrouter.ai](https://openrouter.ai/). Make sure to purchase the credits you need, or sign up for automatic top up.

### 3. Configure Models (Optional)

Edit `src-tauri/src/config.rs` to customize the council:

```rust
pub const COUNCIL_MODELS: &[&str] = &[
    "openai/gpt-oss-20b:free",
    "google/gemma-3-27b-it:free",
    "meta-llama/llama-3.3-70b-instruct:free",
    "x-ai/grok-4.1-fast:free",
    "qwen/qwen3-235b-a22b:free",
    "nousresearch/hermes-3-llama-3.1-405b:free",
    "mistralai/mistral-small-3.1-24b-instruct:free",
    "tngtech/deepseek-r1t2-chimera:free",
];

pub const CHAIRMAN_MODEL: &str = "google/gemini-3-pro-preview";
```

## Running the Application

### Desktop App (Recommended)

Run the Tauri desktop application:
```bash
npm run dev
```

This will:
1. Start the Rust backend on port 8001
2. Start the frontend dev server on port 5173
3. Open the desktop app window

### Web App (Alternative)

**Option 1: Run Rust backend + web frontend**

Terminal 1 (Rust Backend):
```bash
cd src-tauri
cargo run
```

Terminal 2 (Frontend):
```bash
cd frontend
npm run dev
```

Then open http://localhost:5173 in your browser.

**Option 2: Use archived Python backend**

See `backend-python-backup/README.md` for instructions on running the original Python backend.

## Tech Stack

- **Backend:** Rust + Axum (async HTTP framework), tokio runtime, OpenRouter API
- **Desktop:** Tauri 2.x (native desktop app framework)
- **Frontend:** React 19 + Vite, react-markdown for rendering
- **Storage:** JSON files in `data/conversations/`
- **Package Management:** Cargo for Rust, npm for JavaScript

### Original Python Backend

The original Python/FastAPI backend is archived in `backend-python-backup/` and can still be used for:
- Reference implementation
- Regression testing
- Performance comparisons

See `backend-python-backup/README.md` for details.

## Building for Production

```bash
# Build desktop app for your platform
npm run build
```

This generates a distributable app in `src-tauri/target/release/bundle/`:
- **macOS**: `.app` and `.dmg`
- **Windows**: `.msi` installer
- **Linux**: `.AppImage`

## Migration Notes

This project was migrated from Python to Rust in December 2025:
- **Performance**: 2-3x faster startup, lower memory usage
- **Bundle Size**: ~15MB (vs ~50MB+ with Python)
- **API Compatibility**: 100% compatible with original Python backend
- **Conversation Data**: Existing JSON conversations work without modification

See `TAURI_MIGRATION_PLAN.md` for detailed migration information.
