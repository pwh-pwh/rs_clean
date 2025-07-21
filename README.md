# 🧹 rs_clean – Clean Build Targets for Rust, Go, Gradle, and Maven

> ⚡ Easily remove compiled build artifacts from Rust, Go, Gradle, and Maven projects with a single command.

📘 Looking for Chinese docs? [View 中文说明 🇨🇳](./README_zh.md)


## 🚀 Quick Start

```bash
$ rs_clean folder/
````

This command recursively removes build directories in the specified folder and its subdirectories.

---

## 📦 Installation

### Option 1: Install via Cargo

```bash
cargo install rs_clean
```

### Option 2: Download from Releases

👉 [Download from GitHub Releases](https://github.com/your-repo/releases)
Grab the latest binary for your operating system.

---

## ✨ Features

* ✅ Cleans **Rust** projects: `target/`
* ✅ Cleans **Go** build output
* ✅ Cleans **Gradle** projects: `build/`
* ✅ Cleans **Maven** projects: `target/`
* ✅ Recursively scans subdirectories
* ✅ Automatically detects project type

---

## 📂 Example Structure

```bash
$ tree my_projects/
my_projects/
├── rust_app/
│   └── target/
├── go_service/
│   └── bin/
├── gradle_app/
│   └── build/
└── maven_module/
    └── target/
```

After running:

```bash
$ rs_clean my_projects/
```

The build artifacts will be cleaned:

```bash
$ tree my_projects/
my_projects/
├── rust_app/
├── go_service/
├── gradle_app/
└── maven_module/
```

---

## 💡 Use Cases

* Free up disk space by removing large build folders.
* Ensure a clean build environment in CI/CD pipelines.
* Clean multiple types of projects in monorepos.

---

## 🛠 Roadmap

* [ ] Support Node.js projects (`node_modules/`)
* [ ] Show disk space saved after cleanup
* [ ] Add interactive confirmation prompts

---

## 🤝 Contributing

We welcome contributions and feedback!

* Open an [issue](https://github.com/pwh-pwh/rs_clean/issues) for bugs or suggestions
* Submit a pull request for enhancements
* Star ⭐ the repo if you find it helpful

---

## 📄 License

MIT License © 2025 \[coderpwh]
