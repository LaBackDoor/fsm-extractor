# plc-fsm-analyzer

A high-performance Rust tool for extracting, analyzing, and verifying Finite State Machines (FSMs) from PLC XML projects. It implements **SAIN-based state signatures** for industrial anomaly detection.

## 🚀 Features

* **Extraction:** Parses FSM logic from PLC function blocks (IEC 61131-3) into a structured graph.
* **Security Analysis:** Generates "State Signatures" (DNF logic) to verify valid runtime behavior.
* **Validation:** Automatically detects **unreachable states**, **dead-ends**, and **infinite cycles**.
* **Visualization:** Exports diagrams to **Graphviz DOT**, JSON, and Markdown.

## 📦 Installation

```bash
# Build from source
cargo build --release

# Run
./target/release/plc-fsm-analyzer --help