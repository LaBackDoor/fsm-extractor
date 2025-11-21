# 🦀 fsm-extractor: PLC State Machine Extractor

**fsm-extractor** is a **fast, safe, and modular** command-line utility written in **Rust** for extracting and analyzing **Finite State Machines (FSMs)** from proprietary PLC project XML files.

---

## ✨ Key Features

### 1. Core Functionality
* **Complete XML parsing** using `roxmltree` for efficient and zero-copy data access.
* Extracts **function blocks** from `<function-block-declaration>` tags.
* Identifies **case statements** and extracts state machine logic.
* Parses **complex conditional expressions** (e.g., `A AND B OR (C XOR D)`).
* Builds **state transition graphs** from extracted logic.

### 2. Architecture Improvements (Rust Advantage)
* **Modular Design:** Separated into clear modules (`xml_parser`, `fsm`, `analysis`, `output`) for maintainability.
* **Type Safety:** Leverages **Rust's ownership system** for guaranteed memory safety and thread-safe operations.
* **Error Handling:** Comprehensive, user-friendly error types with `anyhow` and `thiserror`.
* **Performance:** Achieves near-native speeds through **zero-cost abstractions** and efficient parsing.

---

## 💻 CLI Interface

The tool provides a powerful command-line interface for various operations.

### Usage Examples

```bash
# Extract FSMs from the input file and output to the terminal (default format: text)
fsm-extractor extract input.xml

# Extract specific function blocks (PRG_VGR_Ablauf, PRG_HBW_Ablauf) to a JSON file
fsm-extractor extract input.xml --format json -f PRG_VGR_Ablauf,PRG_HBW_Ablauf -o output.json

# Analyze all FSM structures for potential issues (dead states, cycles, etc.)
fsm-extractor analyze input.xml --all

# Generate a Graphviz visualization (.dot file) for a specific FSM
fsm-extractor visualize input.xml -o fsm.dot