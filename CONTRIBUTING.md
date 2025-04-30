# Contributing to PCCC

Thank you for your interest in contributing to PCCC! This document provides guidelines for building and working with the documentation.

## Building the Documentation

The documentation is built using [mdBook](https://rust-lang.github.io/mdBook/), a tool for creating books from Markdown files.

### Prerequisites

1. Install mdBook:

   ```bash
   cargo install mdbook
   ```

### Building the Book

1. Navigate to the `docs` directory:

   ```bash
   cd docs
   ```

2. Build the book:

   ```bash
   mdbook build
   ```

   The built book will be available in the `docs/book` directory.

### Previewing the Book

To preview the book while making changes:

```bash
mdbook serve --open
```

This will:

- Build the book
- Start a local web server (default: http://localhost:3000)
- Open the book in your default web browser
- Watch for changes and automatically rebuild when files are modified

## Writing Documentation

When contributing to the documentation:

1. Write in clear, concise English
2. Use proper Markdown formatting
3. Include code examples where appropriate
4. Keep the documentation up-to-date with code changes
5. Follow the existing documentation style and structure

## Submitting Changes

1. Create a new branch for your changes
2. Make your changes to the documentation
3. Build and preview the book to ensure everything looks correct
4. Submit a pull request with a clear description of your changes
