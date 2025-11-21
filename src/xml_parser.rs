use crate::error::FsmError;
use anyhow::Result;
use roxmltree::{Document, Node};
use std::path::Path;
use std::fs;

pub struct XmlParser {
    content: String,
    document: Document<'static>,
}

impl XmlParser {
    pub fn new(xml_path: &Path) -> Result<Self> {
        let content = fs::read_to_string(xml_path)?;
        // Preprocess content similar to C# implementation
        let content = content
            .replace("<expression><integer-literal>", "<value><integer-literal>")
            .replace("<expression><boolean-literal>", "<value><boolean-literal>");

        // This is a workaround for the lifetime issue with roxmltree
        // In production, you'd want to handle this more carefully
        let content_leaked = Box::leak(content.clone().into_boxed_str());
        let document = Document::parse(content_leaked)
            .map_err(|e| FsmError::XmlParse(e.to_string()))?;

        Ok(Self {
            content,
            document,
        })
    }

    pub fn find_function_blocks(&self) -> Vec<String> {
        let mut blocks = Vec::new();

        for node in self.document.descendants() {
            let tag_name = node.tag_name().name();
            if tag_name == "function-block-declaration" {
                if let Some(name) = self.extract_function_block_name(&node) {
                    blocks.push(name);
                }
            } else if tag_name == "program-declaration" {
                if let Some(name) = self.extract_program_name(&node) {
                    blocks.push(name);
                }
            }
        }

        blocks
    }

    pub fn extract_function_block(&self, name: &str) -> Result<FunctionBlockData> {
        let fb_node = self.find_function_block_node(name)
            .ok_or_else(|| FsmError::FunctionBlockNotFound(name.to_string()))?;

        let case_stmt = self.find_case_statement(&fb_node)
            .ok_or_else(|| FsmError::NoCaseStatement(name.to_string()))?;

        let case_variable = self.extract_case_variable(&case_stmt)?;
        let case_elements = self.extract_case_elements(&case_stmt)?;

        Ok(FunctionBlockData {
            name: name.to_string(),
            case_variable,
            case_elements,
        })
    }

    fn find_function_block_node(&self, name: &str) -> Option<Node> {
        for node in self.document.descendants() {
            let tag_name = node.tag_name().name();
            let current_name = if tag_name == "function-block-declaration" {
                self.extract_function_block_name(&node)
            } else if tag_name == "program-declaration" {
                self.extract_program_name(&node)
            } else {
                None
            };

            if let Some(block_name) = current_name {
                if block_name == name {
                    return Some(node);
                }
            }
        }
        None
    }

    fn extract_function_block_name<'a>(&self, fb_node: &Node<'a, '_>) -> Option<String> {
        fb_node.descendants()
            .find(|n| n.tag_name().name() == "derived-function-block-name")
            .and_then(|n| n.text())
            .map(|s| s.to_string())
    }

    fn extract_program_name<'a>(&self, prog_node: &Node<'a, '_>) -> Option<String> {
        prog_node.descendants()
            .find(|n| n.tag_name().name() == "program-type-name")
            .and_then(|n| n.text())
            .map(|s| s.to_string())
    }

    fn find_case_statement<'a>(&self, fb_node: &Node<'a, 'a>) -> Option<Node<'a, 'a>> {
        fb_node.descendants()
            .find(|n| n.tag_name().name() == "case-statement")
    }

    fn extract_case_variable(&self, case_stmt: &Node) -> Result<String> {
        case_stmt.descendants()
            .find(|n| n.tag_name().name() == "variable-name")
            .and_then(|n| n.text())
            .map(|s| s.to_string())
            .ok_or_else(|| FsmError::XmlParse("Case variable not found".to_string()).into())
    }

    fn extract_case_elements(&self, case_stmt: &Node) -> Result<Vec<CaseElement>> {
        let mut elements = Vec::new();

        for node in case_stmt.descendants() {
            if node.tag_name().name() == "case-element" {
                if let Ok(element) = self.parse_case_element(&node) {
                    elements.push(element);
                }
            }
        }

        Ok(elements)
    }

    fn parse_case_element(&self, element_node: &Node) -> Result<CaseElement> {
        let state_id = self.extract_state_id(element_node)?;
        let if_statements = self.extract_if_statements(element_node)?;

        Ok(CaseElement {
            state_id,
            if_statements,
        })
    }

    fn extract_state_id(&self, element_node: &Node) -> Result<String> {
        for node in element_node.descendants() {
            if node.tag_name().name() == "case-list-element" {
                for child in node.descendants() {
                    if child.tag_name().name() == "integer-literal" {
                        if let Some(text) = child.text() {
                            return Ok(text.to_string());
                        }
                    }
                }
            }
        }
        Err(FsmError::XmlParse("State ID not found".to_string()).into())
    }

    fn extract_if_statements(&self, element_node: &Node) -> Result<Vec<IfStatement>> {
        let mut statements = Vec::new();

        for node in element_node.descendants() {
            if node.tag_name().name() == "if-statement" {
                if let Ok(stmt) = self.parse_if_statement(&node) {
                    statements.push(stmt);
                }
            }
        }

        Ok(statements)
    }

    fn parse_if_statement(&self, if_node: &Node) -> Result<IfStatement> {
        let condition = self.extract_expression(if_node)?;
        let assignments = self.extract_assignments(if_node)?;

        Ok(IfStatement {
            condition,
            assignments,
        })
    }

    fn extract_expression(&self, node: &Node) -> Result<String> {
        if let Some(expr_node) = node.descendants()
            .find(|n| n.tag_name().name() == "expression") {
            Ok(self.parse_expression_node(&expr_node))
        } else {
            Ok(String::new())
        }
    }

    fn parse_expression_node(&self, expr_node: &Node) -> String {
        let mut result = String::new();
        let mut in_not = false;

        for node in expr_node.descendants() {
            match node.tag_name().name() {
                "logical-not" => in_not = true,
                "logical-and" => result.push_str(" AND "),
                "logical-or" => result.push_str(" OR "),
                "equal" => result.push_str(" = "),
                "not-equal" => result.push_str(" <> "),
                "less-than" => result.push_str(" < "),
                "less-or-equal" => result.push_str(" <= "),
                "greater-than" => result.push_str(" > "),
                "greater-or-equal" => result.push_str(" >= "),
                "adding" => result.push_str(" + "),
                "subtracting" => result.push_str(" - "),
                "variable-name" => {
                    if let Some(text) = node.text() {
                        if in_not {
                            result.push_str("NOT ");
                            in_not = false;
                        }
                        result.push_str(text);
                    }
                },
                "integer-literal" | "boolean-literal" => {
                    if let Some(text) = node.text() {
                        result.push_str(text);
                    }
                },
                _ => {}
            }
        }

        result.trim().to_string()
    }

    fn extract_assignments(&self, if_node: &Node) -> Result<Vec<Assignment>> {
        let mut assignments = Vec::new();

        for node in if_node.descendants() {
            if node.tag_name().name() == "assignment-statement" {
                if let Ok(assignment) = self.parse_assignment(&node) {
                    assignments.push(assignment);
                }
            }
        }

        Ok(assignments)
    }

    fn parse_assignment(&self, assign_node: &Node) -> Result<Assignment> {
        let variable = assign_node.descendants()
            .find(|n| n.tag_name().name() == "variable-name")
            .and_then(|n| n.text())
            .unwrap_or("")
            .to_string();

        let value = assign_node.descendants()
            .find(|n| n.tag_name().name() == "integer-literal" ||
                n.tag_name().name() == "boolean-literal")
            .and_then(|n| n.text())
            .unwrap_or("")
            .to_string();

        Ok(Assignment { variable, value })
    }
}

#[derive(Debug)]
pub struct FunctionBlockData {
    pub name: String,
    pub case_variable: String,
    pub case_elements: Vec<CaseElement>,
}

#[derive(Debug)]
pub struct CaseElement {
    pub state_id: String,
    pub if_statements: Vec<IfStatement>,
}

#[derive(Debug)]
pub struct IfStatement {
    pub condition: String,
    pub assignments: Vec<Assignment>,
}

#[derive(Debug)]
pub struct Assignment {
    pub variable: String,
    pub value: String,
}