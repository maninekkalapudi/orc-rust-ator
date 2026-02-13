# Local Development Setup for `orc-rust-ator`

This document provides instructions for setting up and running the `orc-rust-ator` project locally on both Windows and Linux environments, without using Docker.

## Prerequisites

### Common for Both Windows and Linux

*   **Rust Toolchain:** The project is written in Rust. You'll need to install the Rust toolchain using `rustup`.
*   **PostgreSQL:** The project uses PostgreSQL as its database. You'll need a running PostgreSQL instance.
*   **`sqlx-cli`:** This command-line tool is used for managing database migrations.

### Windows Specific

*   **C/C++ Build Tools:** Rust often depends on C/C++ build tools (typically from Visual Studio Build Tools).

### Linux Specific

*   **Build Essentials:** You'll need common build tools like `gcc` and `make`.
*   **PostgreSQL Client Libraries:** Development headers for PostgreSQL are required for `sqlx` to compile.

## Step-by-Step Setup

### 1. Install Rust Toolchain

#### Windows
1.  Open PowerShell or Command Prompt **as an administrator**.
2.  Run the following command and follow the on-screen instructions:
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
    During the installation, choose the default option (`1`) for a typical installation. This will also install the necessary C/C++ build tools for Windows (Visual Studio Build Tools). If you encounter issues, you might need to install the "Desktop development with C++" workload from the Visual Studio Installer.
3.  After installation, restart your terminal or run `source "$HOME/.cargo/env"` (if using Git Bash) to update your PATH.
4.  Verify the installation:
    ```bash
    rustc --version
    cargo --version
    ```

#### Linux
1.  Open your terminal.
2.  Run the following command and follow the on-screen instructions:
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
    Choose the default option (`1`) for a typical installation.
3.  After installation, restart your terminal or run `source "$HOME/.cargo/env"` to update your PATH.
4.  Verify the installation:
    ```bash
    rustc --version
    cargo --version
    ```

### 2. Install PostgreSQL

#### Windows
1.  Download the official PostgreSQL installer for Windows from the [PostgreSQL website](https://www.postgresql.org/download/windows/).
2.  Run the installer and follow the prompts. During installation:
    *   You can choose the default components (PostgreSQL Server, pgAdmin 4, Stack Builder, Command Line Tools).
    *   Set a strong password for the `postgres` superuser. Remember this password.
    *   Note the port number (default is `5432`).
3.  After installation, ensure the PostgreSQL server is running (check via the Services application).

#### Linux (Ubuntu/Debian Example)
1.  Update your package list:
    ```bash
    sudo apt update
    ```
2.  Install PostgreSQL server and client libraries:
    ```bash
    sudo apt install postgresql postgresql-client libpq-dev build-essential
    ```
3.  Start and enable the PostgreSQL service:
    ```bash
    sudo systemctl start postgresql
    sudo systemctl enable postgresql
    ```
4.  Verify the service status:
    ```bash
    sudo systemctl status postgresql
    ```

### 3. PostgreSQL Database Setup

You need to create a database and a user for the `orc-rust-ator` project. We'll use the following details:
*   **Database Name:** `orc_rust_ator_db`
*   **User:** `postgres`
*   **Password:** `password` (Choose a strong password in production)

#### Common Steps for Windows and Linux
1.  Open a terminal (Command Prompt/PowerShell on Windows, your preferred terminal on Linux).
2.  Switch to the `postgres` user (Linux only):
    ```bash
    sudo -i -u postgres
    ```
3.  Connect to the `psql` command-line tool:
    ```bash
    psql
    ```
    (On Windows, you might need to navigate to the PostgreSQL `bin` directory first, e.g., `cd "C:\Program Files\PostgreSQL\<version>\bin"` and then run `psql -U postgres`).
4.  Create the database:
    ```sql
    CREATE DATABASE orc_rust_ator_db;
    ```
5.  Create the user (if you want a dedicated user, otherwise you can use `postgres` superuser):
    ```sql
    CREATE USER postgres WITH PASSWORD 'password';
    ```
6.  Grant privileges to the user on the database:
    ```sql
    GRANT ALL PRIVILEGES ON DATABASE orc_rust_ator_db TO postgres;
    ```
7.  Exit `psql`:
    ```sql
    \q
    ```
8.  Exit `postgres` user (Linux only):
    ```bash
    exit
    ```

### 4. Install `sqlx-cli`

This tool is essential for running database migrations.

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

### 5. Set Environment Variables

The project requires the `DATABASE_URL` environment variable to connect to PostgreSQL. It also uses `RUST_LOG` for logging configuration.

#### Windows (Command Prompt)
```bash
set DATABASE_URL=postgresql://postgres:password@localhost:5432/orc_rust_ator_db
set RUST_LOG=info
```

#### Windows (PowerShell)
```powershell
$env:DATABASE_URL="postgresql://postgres:password@localhost:5432/orc_rust_ator_db"
$env:RUST_LOG="info"
```

#### Linux (Bash/Zsh)
```bash
export DATABASE_URL="postgresql://postgres:password@localhost:5432/orc_rust_ator_db"
export RUST_LOG="info"
```
*Note: For persistent environment variables, you might want to add these lines to your shell's profile file (e.g., `.bashrc`, `.zshrc`, or system-wide configuration).* 

### 6. Run Database Migrations

1.  Navigate to the project's root directory in your terminal:
    ```bash
    cd /path/to/orc-rust-ator
    ```
2.  Run the migrations:
    ```bash
    cargo sqlx migrate run
    ```

### 7. Build and Run the Application

1.  Navigate to the project's root directory in your terminal:
    ```bash
    cd /path/to/orc-rust-ator
    ```
2.  Build the project:
    ```bash
    cargo build
    ```
3.  Run the application:
    ```bash
    cargo run
    ```

The application should now be running and accessible, typically on `http://localhost:8080`.
