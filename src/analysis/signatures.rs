use crate::fsm::{FunctionBlock};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// TYPE ALIASES
// ============================================================================

/// Represents a path as a sequence of (state_id, transition_index)
/// The transition_index points to the transition that led TO this state
/// None for initial states (no incoming transition)
type TransitionPath = Vec<(String, Option<usize>)>;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Represents a single atomic condition in a signature (e.g., "H = Input")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Condition {
    pub variable: String,
    pub operator: String,
    pub value: String,
}

impl Condition {
    pub fn new(variable: String, operator: String, value: String) -> Self {
        Self {
            variable,
            operator,
            value,
        }
    }

    /// Format condition as a string (e.g., "H = Input")
    pub fn to_string(&self) -> String {
        format!("{} {} {}", self.variable, self.operator, self.value)
    }
}

/// Boolean expression tree for parsing complex conditions
#[derive(Debug, Clone, PartialEq)]
enum BooleanExpr {
    /// Atomic condition (e.g., "A = 1")
    Atomic(Condition),
    /// Logical AND
    And(Box<BooleanExpr>, Box<BooleanExpr>),
    /// Logical OR
    Or(Box<BooleanExpr>, Box<BooleanExpr>),
    /// Logical NOT
    Not(Box<BooleanExpr>),
}

impl BooleanExpr {
    /// Convert to Disjunctive Normal Form (DNF): (A AND B) OR (C AND D) OR ...
    /// Each inner Vec<Condition> is a conjunction (AND), outer Vec is disjunction (OR)
    fn to_dnf(&self) -> Vec<Vec<Condition>> {
        match self {
            BooleanExpr::Atomic(cond) => vec![vec![cond.clone()]],

            BooleanExpr::And(left, right) => {
                let left_dnf = left.to_dnf();
                let right_dnf = right.to_dnf();

                // Distribute AND over OR: (A OR B) AND (C OR D) = (A AND C) OR (A AND D) OR (B AND C) OR (B AND D)
                let mut result = Vec::new();
                for left_term in &left_dnf {
                    for right_term in &right_dnf {
                        let mut combined = left_term.clone();
                        combined.extend(right_term.clone());
                        result.push(combined);
                    }
                }
                result
            }

            BooleanExpr::Or(left, right) => {
                let mut left_dnf = left.to_dnf();
                let mut right_dnf = right.to_dnf();
                left_dnf.append(&mut right_dnf);
                left_dnf
            }

            BooleanExpr::Not(inner) => {
                // For NOT, we need to apply De Morgan's laws
                // NOT(A AND B) = NOT(A) OR NOT(B)
                // NOT(A OR B) = NOT(A) AND NOT(B)
                // NOT(NOT(A)) = A
                match inner.as_ref() {
                    BooleanExpr::Atomic(cond) => {
                        // Negate the operator
                        let negated = Self::negate_condition(cond);
                        vec![vec![negated]]
                    }
                    BooleanExpr::And(left, right) => {
                        // NOT(A AND B) = NOT(A) OR NOT(B)
                        let not_left = BooleanExpr::Not(left.clone());
                        let not_right = BooleanExpr::Not(right.clone());
                        BooleanExpr::Or(Box::new(not_left), Box::new(not_right)).to_dnf()
                    }
                    BooleanExpr::Or(left, right) => {
                        // NOT(A OR B) = NOT(A) AND NOT(B)
                        let not_left = BooleanExpr::Not(left.clone());
                        let not_right = BooleanExpr::Not(right.clone());
                        BooleanExpr::And(Box::new(not_left), Box::new(not_right)).to_dnf()
                    }
                    BooleanExpr::Not(inner) => {
                        // NOT(NOT(A)) = A
                        inner.to_dnf()
                    }
                }
            }
        }
    }

    /// Negate a condition operator
    fn negate_condition(cond: &Condition) -> Condition {
        let negated_op = match cond.operator.as_str() {
            "=" => "<>",
            "<>" => "=",
            "<" => ">=",
            "<=" => ">",
            ">" => "<=",
            ">=" => "<",
            _ => "=", // fallback
        };

        Condition::new(
            cond.variable.clone(),
            negated_op.to_string(),
            cond.value.clone(),
        )
    }
}

/// A single path signature (one way to reach a state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSignature {
    pub conditions: Vec<Condition>,
    pub path_id: usize,
}

impl PathSignature {
    pub fn new(conditions: Vec<Condition>, path_id: usize) -> Self {
        Self {
            conditions,
            path_id,
        }
    }

    /// Format signature as a readable string
    pub fn format_conditions(&self) -> String {
        if self.conditions.is_empty() {
            "[initial]".to_string()
        } else {
            self.conditions
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(" AND ")
        }
    }

    /// Check if runtime conditions match this signature
    pub fn matches(&self, runtime_vars: &HashMap<String, String>) -> bool {
        self.conditions.iter().all(|cond| {
            if let Some(runtime_value) = runtime_vars.get(&cond.variable) {
                Self::evaluate_condition(cond, runtime_value)
            } else {
                false // Variable not present in runtime state
            }
        })
    }

    fn evaluate_condition(cond: &Condition, runtime_value: &str) -> bool {
        match cond.operator.as_str() {
            "=" => runtime_value == cond.value,
            "<>" => runtime_value != cond.value,
            "<" => {
                if let (Ok(rv), Ok(cv)) = (runtime_value.parse::<f64>(), cond.value.parse::<f64>()) {
                    rv < cv
                } else {
                    false
                }
            }
            "<=" => {
                if let (Ok(rv), Ok(cv)) = (runtime_value.parse::<f64>(), cond.value.parse::<f64>()) {
                    rv <= cv
                } else {
                    false
                }
            }
            ">" => {
                if let (Ok(rv), Ok(cv)) = (runtime_value.parse::<f64>(), cond.value.parse::<f64>()) {
                    rv > cv
                } else {
                    false
                }
            }
            ">=" => {
                if let (Ok(rv), Ok(cv)) = (runtime_value.parse::<f64>(), cond.value.parse::<f64>()) {
                    rv >= cv
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

/// Signature for a single state (MULTIPLE path signatures - disjunctive OR)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSignature {
    pub state_id: String,
    pub path_signatures: Vec<PathSignature>,
    pub paths_count: usize,
}

impl StateSignature {
    pub fn new(state_id: String) -> Self {
        Self {
            state_id,
            path_signatures: Vec::new(),
            paths_count: 0,
        }
    }

    /// Format all signatures (showing OR logic)
    pub fn format_conditions(&self) -> String {
        if self.path_signatures.is_empty() {
            "[initial]".to_string()
        } else if self.path_signatures.len() == 1 {
            self.path_signatures[0].format_conditions()
        } else {
            // Multiple paths - show as disjunction
            self.path_signatures
                .iter()
                .map(|ps| format!("({})", ps.format_conditions()))
                .collect::<Vec<_>>()
                .join(" OR ")
        }
    }

    /// Check if runtime state matches ANY of the path signatures
    pub fn matches_any(&self, runtime_vars: &HashMap<String, String>) -> bool {
        if self.path_signatures.is_empty() {
            return true; // Initial state
        }
        self.path_signatures.iter().any(|ps| ps.matches(runtime_vars))
    }
}

/// Table of all state signatures for a function block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSignatureTable {
    pub function_block_name: String,
    pub case_variable: String,
    pub signatures: IndexMap<String, StateSignature>,
}

impl StateSignatureTable {
    pub fn new(function_block_name: String, case_variable: String) -> Self {
        Self {
            function_block_name,
            case_variable,
            signatures: IndexMap::new(),
        }
    }

    /// Get signature for a specific state
    pub fn get_signature(&self, state_id: &str) -> Option<&StateSignature> {
        self.signatures.get(state_id)
    }

    /// Total number of states with signatures
    pub fn state_count(&self) -> usize {
        self.signatures.len()
    }

    /// Verify runtime state against signatures (for runtime monitoring)
    pub fn verify_state(&self, state_id: &str, runtime_vars: &HashMap<String, String>) -> bool {
        if let Some(sig) = self.signatures.get(state_id) {
            sig.matches_any(runtime_vars)
        } else {
            false // Unknown state
        }
    }
}

// ============================================================================
// EXPRESSION TOKENIZER
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Condition(String),
    And,
    Or,
    Not,
    LParen,
    RParen,
}

struct Tokenizer {
    input: String,
    position: usize,
}

impl Tokenizer {
    fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            position: 0,
        }
    }

    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while self.position < self.input.len() {
            self.skip_whitespace();

            if self.position >= self.input.len() {
                break;
            }

            // Check for keywords and operators
            if self.check_keyword("AND") {
                tokens.push(Token::And);
                self.position += 3;
            } else if self.check_keyword("OR") {
                tokens.push(Token::Or);
                self.position += 2;
            } else if self.check_keyword("NOT") {
                tokens.push(Token::Not);
                self.position += 3;
            } else if self.current_char() == '(' {
                tokens.push(Token::LParen);
                self.position += 1;
            } else if self.current_char() == ')' {
                tokens.push(Token::RParen);
                self.position += 1;
            } else {
                // Parse atomic condition
                if let Some(condition_str) = self.parse_atomic_condition() {
                    tokens.push(Token::Condition(condition_str));
                } else {
                    // Skip unrecognized character
                    self.position += 1;
                }
            }
        }

        tokens
    }

    fn current_char(&self) -> char {
        self.input.chars().nth(self.position).unwrap_or('\0')
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.current_char().is_whitespace() {
            self.position += 1;
        }
    }

    fn check_keyword(&self, keyword: &str) -> bool {
        if self.position + keyword.len() > self.input.len() {
            return false;
        }

        let substr = &self.input[self.position..self.position + keyword.len()];
        if substr != keyword {
            return false;
        }

        // Ensure it's a complete word (not part of a variable name)
        let next_pos = self.position + keyword.len();
        if next_pos < self.input.len() {
            let next_char = self.input.chars().nth(next_pos).unwrap_or('\0');
            if next_char.is_alphanumeric() || next_char == '_' {
                return false;
            }
        }

        true
    }

    fn parse_atomic_condition(&mut self) -> Option<String> {
        let start = self.position;
        let mut paren_depth = 0;

        // Parse until we hit AND, OR, or unmatched closing paren
        while self.position < self.input.len() {
            let ch = self.current_char();

            if ch == '(' {
                paren_depth += 1;
                self.position += 1;
            } else if ch == ')' {
                if paren_depth > 0 {
                    paren_depth -= 1;
                    self.position += 1;
                } else {
                    // Unmatched closing paren - end of condition
                    break;
                }
            } else if paren_depth == 0 && (self.check_keyword("AND") || self.check_keyword("OR")) {
                break;
            } else {
                self.position += 1;
            }
        }

        if self.position > start {
            let condition = self.input[start..self.position].trim().to_string();
            if !condition.is_empty() {
                return Some(condition);
            }
        }

        None
    }
}

// ============================================================================
// EXPRESSION PARSER
// ============================================================================

/// Parse a single atomic condition expression
/// Handles: =, <>, <=, >=, <, >
fn parse_atomic_condition_str(expr: &str) -> Option<Condition> {
    // Remove outer parentheses if present
    let expr = expr.trim();
    let expr = if expr.starts_with('(') && expr.ends_with(')') {
        &expr[1..expr.len() - 1]
    } else {
        expr
    };

    // Try different operators in order of precedence (longest first)
    let operators = vec![
        ("<=", "<="),
        (">=", ">="),
        ("<>", "<>"),
        ("=", "="),
        ("<", "<"),
        (">", ">"),
    ];

    for (op_str, op_name) in operators {
        if let Some(pos) = expr.find(op_str) {
            let variable = expr[..pos].trim();
            let value = expr[pos + op_str.len()..].trim();

            // Clean up value expressions (remove outer parentheses if they wrap entire value)
            let value = if value.starts_with('(') && value.ends_with(')') {
                value[1..value.len() - 1].trim().to_string()
            } else {
                value.to_string()
            };

            return Some(Condition::new(
                variable.to_string(),
                op_name.to_string(),
                value,
            ));
        }
    }

    None
}

struct ExpressionParser {
    tokens: Vec<Token>,
    position: usize,
}

impl ExpressionParser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn parse(&mut self) -> Option<BooleanExpr> {
        self.parse_or()
    }

    // OR has the lowest precedence
    fn parse_or(&mut self) -> Option<BooleanExpr> {
        let mut left = self.parse_and()?;

        while self.position < self.tokens.len() {
            if matches!(self.tokens[self.position], Token::Or) {
                self.position += 1;
                let right = self.parse_and()?;
                left = BooleanExpr::Or(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }

        Some(left)
    }

    // AND has the higher precedence than OR
    fn parse_and(&mut self) -> Option<BooleanExpr> {
        let mut left = self.parse_not()?;

        while self.position < self.tokens.len() {
            if matches!(self.tokens[self.position], Token::And) {
                self.position += 1;
                let right = self.parse_not()?;
                left = BooleanExpr::And(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }

        Some(left)
    }

    // NOT has the highest precedence
    fn parse_not(&mut self) -> Option<BooleanExpr> {
        if self.position < self.tokens.len() {
            if matches!(self.tokens[self.position], Token::Not) {
                self.position += 1;
                let inner = self.parse_primary()?;
                return Some(BooleanExpr::Not(Box::new(inner)));
            }
        }

        self.parse_primary()
    }

    // Primary expression: atomic condition or parenthesized expression
    fn parse_primary(&mut self) -> Option<BooleanExpr> {
        if self.position >= self.tokens.len() {
            return None;
        }

        match &self.tokens[self.position] {
            Token::LParen => {
                self.position += 1;
                let expr = self.parse_or()?;

                // Expect closing paren
                if self.position < self.tokens.len() && matches!(self.tokens[self.position], Token::RParen) {
                    self.position += 1;
                }

                Some(expr)
            }
            Token::Condition(cond_str) => {
                self.position += 1;
                // Parse the atomic condition using standalone function
                parse_atomic_condition_str(cond_str)
                    .map(BooleanExpr::Atomic)
            }
            _ => None,
        }
    }
}

// ============================================================================
// PATH FINDING
// ============================================================================

pub struct PathFinder;

impl PathFinder {
    /// Find all paths from initial states to each state using DFS
    pub fn find_all_paths(fsm: &FunctionBlock) -> HashMap<String, Vec<TransitionPath>> {
        let mut paths_to_states: HashMap<String, Vec<TransitionPath>> = HashMap::new();
        let initial_states = Self::find_initial_states(fsm);

        let starting_states = if initial_states.is_empty() {
            Self::find_fallback_initial_state(fsm)
        } else {
            initial_states
        };

        for initial in starting_states {
            let mut visited = HashSet::new();
            let mut current_path = vec![(initial.clone(), None)];
            Self::dfs(
                fsm,
                &initial,
                &mut visited,
                &mut current_path,
                &mut paths_to_states,
            );
        }

        paths_to_states
    }

    fn find_initial_states(fsm: &FunctionBlock) -> Vec<String> {
        fsm.states
            .values()
            .filter(|s| s.transitions_in.is_empty())
            .map(|s| s.id.clone())
            .collect()
    }

    fn find_fallback_initial_state(fsm: &FunctionBlock) -> Vec<String> {
        if fsm.states.contains_key("100") {
            vec!["100".to_string()]
        } else if fsm.states.contains_key("10") {
            vec!["10".to_string()]
        } else if let Some(first_state) = fsm.states.keys().next() {
            vec![first_state.clone()]
        } else {
            Vec::new()
        }
    }

    fn dfs(
        fsm: &FunctionBlock,
        current_state: &str,
        visited: &mut HashSet<String>,
        current_path: &mut TransitionPath,
        paths_to_states: &mut HashMap<String, Vec<TransitionPath>>,
    ) {
        paths_to_states
            .entry(current_state.to_string())
            .or_insert_with(Vec::new)
            .push(current_path.clone());

        visited.insert(current_state.to_string());

        for (trans_idx, transition) in fsm.transitions.iter().enumerate() {
            if transition.from_state == current_state {
                let next_state = &transition.to_state;

                if !visited.contains(next_state) {
                    current_path.push((next_state.clone(), Some(trans_idx)));
                    Self::dfs(fsm, next_state, visited, current_path, paths_to_states);
                    current_path.pop();
                }
            }
        }

        visited.remove(current_state);
    }
}

// ============================================================================
// SIGNATURE GENERATION
// ============================================================================

pub struct SignatureGenerator;

impl SignatureGenerator {
    pub fn generate(fsm: &FunctionBlock) -> StateSignatureTable {
        let mut table = StateSignatureTable::new(fsm.name.clone(), fsm.case_variable.clone());
        let paths = PathFinder::find_all_paths(fsm);

        for (state_id, paths_to_state) in paths {
            let signature = Self::build_signature_for_state(fsm, &state_id, &paths_to_state);
            table.signatures.insert(state_id.clone(), signature);
        }

        table
    }

    fn build_signature_for_state(
        fsm: &FunctionBlock,
        state_id: &str,
        paths: &[TransitionPath],
    ) -> StateSignature {
        let mut path_signatures = Vec::new();
        let mut signature_id = 0;

        for path in paths.iter() {
            let condition_sets = Self::extract_conditions_from_path(fsm, path);

            for conditions in condition_sets {
                let unique_conditions = Self::remove_redundancy_in_path(conditions);
                path_signatures.push(PathSignature::new(unique_conditions, signature_id));
                signature_id += 1;
            }
        }

        let optimized_signatures = Self::merge_equivalent_signatures(path_signatures);

        StateSignature {
            state_id: state_id.to_string(),
            path_signatures: optimized_signatures,
            paths_count: paths.len(),
        }
    }

    fn extract_conditions_from_path(fsm: &FunctionBlock, path: &TransitionPath) -> Vec<Vec<Condition>> {
        let mut transition_dnfs: Vec<Vec<Vec<Condition>>> = Vec::new();

        for (_state_id, transition_idx) in path {
            if let Some(idx) = transition_idx {
                if let Some(transition) = fsm.transitions.get(*idx) {
                    let dnf = Self::parse_transition_condition(&transition.condition);
                    transition_dnfs.push(dnf);
                }
            }
        }

        Self::cross_product_dnf(transition_dnfs)
    }

    fn cross_product_dnf(dnfs: Vec<Vec<Vec<Condition>>>) -> Vec<Vec<Condition>> {
        if dnfs.is_empty() {
            return vec![vec![]];
        }

        let mut result = dnfs[0].clone();

        for dnf in dnfs.iter().skip(1) {
            let mut new_result = Vec::new();

            for left_term in &result {
                for right_term in dnf {
                    let mut combined = left_term.clone();
                    combined.extend(right_term.clone());
                    new_result.push(combined);
                }
            }

            result = new_result;
        }

        result
    }

    fn parse_transition_condition(condition_str: &str) -> Vec<Vec<Condition>> {
        if condition_str.is_empty() || condition_str == "No Check" {
            return vec![vec![]];
        }

        let mut tokenizer = Tokenizer::new(condition_str);
        let tokens = tokenizer.tokenize();

        if tokens.is_empty() {
            return vec![vec![]];
        }

        let mut parser = ExpressionParser::new(tokens);
        let expr = match parser.parse() {
            Some(e) => e,
            None => {
                return Self::parse_simple_condition(condition_str);
            }
        };

        let dnf = expr.to_dnf();

        dnf.into_iter()
            .map(|conjunction| {
                let mut seen = HashSet::new();
                let mut unique = Vec::new();
                for cond in conjunction {
                    let key = (cond.variable.clone(), cond.operator.clone(), cond.value.clone());
                    if seen.insert(key) {
                        unique.push(cond);
                    }
                }
                unique
            })
            .collect()
    }

    fn parse_simple_condition(condition_str: &str) -> Vec<Vec<Condition>> {
        let mut conditions = Vec::new();

        let parts: Vec<&str> = condition_str.split(" AND ").collect();

        for part in parts {
            let trimmed = part.trim();
            if let Some(cond) = Self::parse_single_condition(trimmed) {
                conditions.push(cond);
            }
        }

        vec![conditions]
    }

    fn parse_single_condition(expr: &str) -> Option<Condition> {
        parse_atomic_condition_str(expr)
    }

    fn remove_redundancy_in_path(conditions: Vec<Condition>) -> Vec<Condition> {
        let mut seen = HashSet::new();
        let mut unique = Vec::new();

        for cond in conditions {
            if seen.insert((cond.variable.clone(), cond.operator.clone(), cond.value.clone())) {
                unique.push(cond);
            }
        }

        unique.sort_by(|a, b| {
            a.variable
                .cmp(&b.variable)
                .then_with(|| a.operator.cmp(&b.operator))
                .then_with(|| a.value.cmp(&b.value))
        });

        unique
    }

    fn merge_equivalent_signatures(mut signatures: Vec<PathSignature>) -> Vec<PathSignature> {
        if signatures.len() <= 1 {
            return signatures;
        }

        let mut grouped: HashMap<String, PathSignature> = HashMap::new();

        for sig in signatures {
            let key = sig.format_conditions();
            grouped.entry(key).or_insert(sig);
        }

        grouped.into_values().collect()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsm::{FunctionBlock, State, Transition};

    fn create_test_fsm() -> FunctionBlock {
        let mut fb = FunctionBlock::new("TestFB".to_string(), "state".to_string());
        fb.add_state(State::new("10".to_string()));
        fb.add_state(State::new("20".to_string()));
        fb.add_state(State::new("30".to_string()));
        fb.add_transition(Transition::new("10".to_string(), "20".to_string(), "sensor = low".to_string()));
        fb.add_transition(Transition::new("20".to_string(), "30".to_string(), "sensor = high".to_string()));
        fb
    }

    fn create_cyclic_fsm() -> FunctionBlock {
        let mut fb = FunctionBlock::new("CyclicFB".to_string(), "state".to_string());
        fb.add_state(State::new("10".to_string()));
        fb.add_state(State::new("20".to_string()));
        fb.add_state(State::new("30".to_string()));
        fb.add_transition(Transition::new("10".to_string(), "20".to_string(), "sensor = low".to_string()));
        fb.add_transition(Transition::new("20".to_string(), "30".to_string(), "sensor = high".to_string()));
        fb.add_transition(Transition::new("30".to_string(), "10".to_string(), "reset = true".to_string()));
        fb
    }

    fn create_multi_path_fsm() -> FunctionBlock {
        let mut fb = FunctionBlock::new("MultiPathFB".to_string(), "state".to_string());
        fb.add_state(State::new("10".to_string()));
        fb.add_state(State::new("20".to_string()));
        fb.add_state(State::new("30".to_string()));
        fb.add_transition(Transition::new("10".to_string(), "20".to_string(), "sensor = low".to_string()));
        fb.add_transition(Transition::new("10".to_string(), "20".to_string(), "button = pressed".to_string()));
        fb.add_transition(Transition::new("20".to_string(), "30".to_string(), "timer > 100".to_string()));
        fb
    }

    #[test]
    fn test_multiple_path_signatures() {
        let fsm = create_multi_path_fsm();
        let table = SignatureGenerator::generate(&fsm);
        let sig_20 = table.get_signature("20").unwrap();
        assert_eq!(sig_20.path_signatures.len(), 2);
    }

    #[test]
    fn test_runtime_verification() {
        let fsm = create_multi_path_fsm();
        let table = SignatureGenerator::generate(&fsm);

        let mut runtime_vars_a = HashMap::new();
        runtime_vars_a.insert("sensor".to_string(), "low".to_string());
        assert!(table.verify_state("20", &runtime_vars_a));

        let mut runtime_vars_b = HashMap::new();
        runtime_vars_b.insert("button".to_string(), "pressed".to_string());
        assert!(table.verify_state("20", &runtime_vars_b));

        let runtime_vars_c = HashMap::new();
        assert!(!table.verify_state("20", &runtime_vars_c));
    }

    #[test]
    fn test_parse_simple_and() {
        let dnf = SignatureGenerator::parse_transition_condition("A = 1 AND B = 2");
        assert_eq!(dnf.len(), 1);
        assert_eq!(dnf[0].len(), 2);
    }

    #[test]
    fn test_parse_simple_or() {
        let dnf = SignatureGenerator::parse_transition_condition("A = 1 OR B = 2");
        assert_eq!(dnf.len(), 2);
    }

    #[test]
    fn test_parse_complex_and_or() {
        let dnf = SignatureGenerator::parse_transition_condition("(A = 1 OR B = 2) AND C = 3");
        assert_eq!(dnf.len(), 2);
    }

    #[test]
    fn test_fsm_with_or_condition() {
        let mut fb = FunctionBlock::new("OrTestFB".to_string(), "state".to_string());
        fb.add_state(State::new("10".to_string()));
        fb.add_state(State::new("20".to_string()));
        fb.add_transition(Transition::new("10".to_string(), "20".to_string(), "sensor = low OR button = pressed".to_string()));

        let table = SignatureGenerator::generate(&fb);
        let sig_20 = table.get_signature("20").unwrap();
        assert_eq!(sig_20.path_signatures.len(), 2);
    }
}