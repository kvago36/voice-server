# Project Installation Guide

## Prerequisites
- Ensure you have **Git** and **Rust** installed on your system.
- You need to clone two repositories into the `proto/` directory: **Yandex Cloud** and **Google APIs**.

## Installation Steps

### 1. Clone the required repositories
```sh
mkdir -p proto
cd proto

git clone https://github.com/yandex-cloud/api.git

git clone https://github.com/googleapis/googleapis.git
```

### 2. Build the project
Navigate to the root of the project and build it using Cargo:

```sh
cd ..  # Return to the root directory
cargo build
```

### 3. Run the project
To start the application, run:

```sh
cargo run
```

## Notes
- Ensure that the `proto/` directory contains both cloned repositories.
- If you encounter any issues with dependencies, try running:

```sh
cargo clean
cargo build
```

## Contributing
Feel free to submit pull requests or report issues.

## License
This project is licensed under the MIT License.

