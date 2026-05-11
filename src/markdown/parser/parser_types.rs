//! Markdown parser data model types.

/// Heading level enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HeadingLevel {
    H1 = 1,
    H2 = 2,
    H3 = 3,
    H4 = 4,
    H5 = 5,
    H6 = 6,
}

impl HeadingLevel {
    /// Create from a numeric level (1-6)
    pub fn from_usize(level: usize) -> Option<Self> {
        match level {
            1 => Some(HeadingLevel::H1),
            2 => Some(HeadingLevel::H2),
            3 => Some(HeadingLevel::H3),
            4 => Some(HeadingLevel::H4),
            5 => Some(HeadingLevel::H5),
            6 => Some(HeadingLevel::H6),
            _ => None,
        }
    }

    /// Get the numeric level
    pub fn as_usize(&self) -> usize {
        *self as usize
    }

    /// Check if this level can follow another level
    pub fn can_follow(&self, previous: Option<HeadingLevel>) -> bool {
        match previous {
            None => *self == HeadingLevel::H1,
            Some(prev) => {
                let prev_level = prev.as_usize();
                let this_level = self.as_usize();
                // Can be same level, one level deeper, or any level up
                this_level >= 1 && this_level <= prev_level + 1
            }
        }
    }
}

impl std::fmt::Display for HeadingLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "H{}", self.as_usize())
    }
}

/// Represents a heading in a markdown document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// Heading level (1-6)
    pub level: HeadingLevel,
    /// Heading text content
    pub text: String,
    /// Position in the document body
    pub position: usize,
    /// Line number in the source
    pub line_number: usize,
}

/// Represents a link in a markdown document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// Link URL
    pub url: String,
    /// Link text
    pub text: String,
    /// Line number in the source
    pub line_number: usize,
}

/// Represents a code block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeBlock {
    /// Language identifier (if any)
    pub language: Option<String>,
    /// Code content
    pub content: String,
    /// Line number in the source
    pub line_number: usize,
}

/// Hierarchical structure of headings
#[derive(Debug, Clone)]
pub struct HeadingHierarchy {
    pub nodes: Vec<HeadingNode>,
}

impl HeadingHierarchy {
    pub(super) fn from_headings(headings: &[Heading]) -> Self {
        let mut nodes = Vec::new();
        let mut stack: Vec<(HeadingLevel, Vec<HeadingNode>)> = Vec::new();

        for heading in headings {
            let node = HeadingNode {
                heading: heading.clone(),
                children: Vec::new(),
            };

            // Process Section 2 (H2): H2 <= H3, so pop H3 first
            // Pop (H3, [SubsectionNode]), attach to Section 1
            // Now H2 <= H2, pop (H2, [Section1Node_with_Subsection])
            // Attach Section 1 to Title, push (H1, [TitleNode_with_Section1])
            // Now H2 > H1, push (H2, [Section2Node])
            // Stack: [(H1, [TitleNode_with_Section1]), (H2, [Section2Node])]

            // When processing Section 2 (H2), the algorithm pops until it finds a parent
            // with level < H2, which is H1. But the issue is that when it pops Section 1
            // and attaches it to Title, it replaces Title's children instead of extending them.

            // After flush:
            // Pop (H2, [Section2Node]), attach to Title, replacing Section 1
            // Result: Title has only Section 2 as child

            // The fix is to extend children instead of replacing them.

            // Pop stack until we find the parent or stack is empty
            while let Some((level, _)) = stack.last() {
                if heading.level.as_usize() > level.as_usize() {
                    break;
                }
                let (_, children) = stack.pop().unwrap();
                if let Some((_, parent_children)) = stack.last_mut() {
                    if let Some(mut parent) = parent_children.pop() {
                        parent.children.extend(children); // Extend instead of replace
                        parent_children.push(parent);
                    }
                } else {
                    // No parent, these are top-level nodes
                    nodes.extend(children);
                }
            }

            // Push current node
            stack.push((heading.level, vec![node]));
        }

        // Flush remaining stack
        while let Some((_, children)) = stack.pop() {
            if let Some((_, parent_children)) = stack.last_mut() {
                if let Some(mut parent) = parent_children.pop() {
                    parent.children.extend(children); // Extend instead of replace
                    parent_children.push(parent);
                }
            } else {
                nodes.extend(children);
            }
        }

        Self { nodes }
    }

    /// Check if the hierarchy is valid (no skipped levels)
    pub fn is_valid(&self) -> bool {
        for node in &self.nodes {
            if node.heading.level != HeadingLevel::H1 {
                return false;
            }
            if !Self::check_children_valid(&node.children, HeadingLevel::H1) {
                return false;
            }
        }
        true
    }

    fn check_children_valid(children: &[HeadingNode], parent_level: HeadingLevel) -> bool {
        for child in children {
            let level_diff = child.heading.level.as_usize() - parent_level.as_usize();
            // Children must be exactly one level deeper
            if level_diff != 1 {
                return false;
            }
            if !Self::check_children_valid(&child.children, child.heading.level) {
                return false;
            }
        }
        true
    }

    /// Get all heading texts at a specific level
    pub fn get_headings_at_level(&self, level: HeadingLevel) -> Vec<String> {
        let mut result = Vec::new();
        for node in &self.nodes {
            Self::collect_headings_at_level(node, level, &mut result);
        }
        result
    }

    fn collect_headings_at_level(
        node: &HeadingNode,
        level: HeadingLevel,
        result: &mut Vec<String>,
    ) {
        if node.heading.level == level {
            result.push(node.heading.text.clone());
        }
        for child in &node.children {
            Self::collect_headings_at_level(child, level, result);
        }
    }
}

/// A node in the heading hierarchy
#[derive(Debug, Clone)]
pub struct HeadingNode {
    pub heading: Heading,
    pub children: Vec<HeadingNode>,
}
