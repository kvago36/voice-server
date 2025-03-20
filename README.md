
# Voice Server

**Voice Server** is a server-side application written in **Rust** for processing and managing voice data using **WebSocket (actix-ws)**, **PostgreSQL**, and **gRPC (tonic)**.

## Description

The project implements a WebSocket server for receiving audio chunks from clients, as well as a REST API for managing processing results. Speech recognition services are integrated through the **Yandex Cloud** and **Google** APIs.

### Key Components:
- **Actix-web** – Actix Web is a powerful, pragmatic, and extremely fast web framework for Rust.
- **Tonic** – A Rust implementation of gRPC, a high-performance, open-source, general RPC framework that prioritizes mobile and HTTP/2.
- **sqlx** – The async SQL toolkit for Rust.

## Installation

### Prerequisites
- Installed **Git**, **Rust**, **PostgreSQL**.
- A `.env` configuration file with database connection parameters.
- Alternatively, using docker-compose.

### Installation Steps

1. **Clone the necessary repositories**
   ```bash
   mkdir -p proto
   cd proto
   git clone https://github.com/yandex-cloud/api.git
   git clone https://github.com/googleapis/googleapis.git
   cd ..
   ```
    
2. **Set up the database**
   ```sql
   CREATE DATABASE postgres;
   ```

3. **Set up environment variables**
   Create a `.env` file in the root directory:
   ```ini
   DATABASE_URL=postgresql://postgres:example@localhost:5432/postgres
   YA_CLOUD_URL=your_cloud_url
   OAUTH_TOKEN=your_oauth_token
   FOLDER_ID=your_folder_id
   ```

   For more information on how to obtain the **OAuth Token** and **Folder ID**, please refer to the [Yandex Cloud FAQ](https://cloud.yandex.com/docs/iam/quickstart).


4. **Build and run the project**
   ```bash
   cargo build
   cargo run
   ```

## Usage

After running:
- **WebSocket connection** is available at `ws://localhost:8000/ws`.
- **REST API** is available at `http://localhost:PORT/8000`.

## Technologies and Dependencies

| Library          | Purpose |
|------------------|---------------------------------|
| **actix-web**    | REST API, WebSocket server |
| **actix-ws**     | WebSocket connection support |
| **tonic**        | gRPC client for working with APIs |
| **prost**        | Protocol Buffers serialization |
| **serde**        | Data serialization to JSON |
| **serde_json**   | Working with JSON |
| **sqlx**         | Asynchronous PostgreSQL interaction |
| **dotenv**       | Loading environment variables |
| **thiserror**    | Custom error definitions |
| **log**          | Logging |
| **env_logger**   | Logging configuration via environment |
