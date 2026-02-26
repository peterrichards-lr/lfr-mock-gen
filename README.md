## Liferay Mock Content Generator (Gemini Integration)

This repository includes a powerful CLI tool built in Rust that automatically generates realistic mock data for Liferay using the Gemini AI API. It creates 20 Basic Web Content articles, populates them with unique AI-generated titles, descriptions, and body paragraphs, and injects open-source placeholder images.

The tool uses Liferay's Headless Delivery REST APIs to push the content directly into your chosen Site, completely bypassing the need for complex `.lar` file imports or Groovy scripts.

### Prerequisites

1. **Rust and Cargo**: Ensure you have the latest stable version of Rust installed.
2. **Gemini API Key**: You will need a free API key from Google AI Studio.
3. **Liferay DXP/CE**: A running instance of Liferay 7.3+ (requires the Headless Delivery APIs).

### Installation & Build

Compile the application for release to ensure maximum performance:

```bash
cargo build --release
